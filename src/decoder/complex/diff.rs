use super::missing::DecodedValue::{self, Normal};
use crate::{
    decoder::{param::SpatialDifferencingParam, DecodeError},
    helpers::grib_int_from_bytes,
};

pub(crate) struct SpatialDifferencingExtraDescriptors<'a> {
    slice: &'a [u8],
    num_octets: usize,
}

impl<'a> SpatialDifferencingExtraDescriptors<'a> {
    pub(crate) fn new(
        param: &SpatialDifferencingParam,
        parent_slice: &'a [u8],
    ) -> Result<Self, DecodeError> {
        let SpatialDifferencingParam {
            order,
            extra_desc_num_octets,
        } = param;
        if *extra_desc_num_octets == 0 || *extra_desc_num_octets > 4 {
            return Err(DecodeError::from(
                format!("unexpected value for \"number of octets required in the data section to specify extra descriptors needed for spatial differencing\": {extra_desc_num_octets}")
            ));
        }
        let num_octets = usize::from(*extra_desc_num_octets);
        let byte_length = usize::from(order + 1) * num_octets;

        Ok(Self {
            slice: &parent_slice[..byte_length],
            num_octets,
        })
    }

    // total number of octets for descriptors
    pub(crate) fn len(&self) -> usize {
        self.slice.len()
    }

    // overall minimum of the differences
    pub(crate) fn minimum(&self) -> i32 {
        let slice = &self.slice[self.first_value_end_pos()..];
        grib_int_from_bytes(slice)
    }

    pub(crate) fn first_values(&self) -> FirstValues<'_, 'a> {
        FirstValues::new(self)
    }

    fn first_value_end_pos(&self) -> usize {
        self.len() - self.num_octets
    }
}

pub(crate) struct FirstValues<'s, 'a> {
    spdiff_info: &'s SpatialDifferencingExtraDescriptors<'a>,
    pos: usize,
}

impl<'s, 'a> FirstValues<'s, 'a> {
    pub(crate) fn new(spdiff_info: &'s SpatialDifferencingExtraDescriptors<'a>) -> Self {
        Self {
            spdiff_info,
            pos: 0,
        }
    }
}

impl<'s, 'a> Iterator for FirstValues<'s, 'a> {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.spdiff_info.first_value_end_pos() {
            return None;
        }

        let num_octets = self.spdiff_info.num_octets;
        let slice = &self.spdiff_info.slice[self.pos..self.pos + num_octets];
        let val = grib_int_from_bytes(slice);
        self.pos += num_octets;
        Some(val)
    }
}

pub(crate) enum SpatialDifferencingDecodeIterator<I, J> {
    FirstOrder(FirstOrderSpatialDifferencingDecodeIterator<I, J>),
    SecondOrder(SecondOrderSpatialDifferencingDecodeIterator<I, J>),
}

impl<I, J> Iterator for SpatialDifferencingDecodeIterator<I, J>
where
    I: Iterator<Item = DecodedValue<i32>>,
    J: Iterator<Item = i32>,
{
    type Item = DecodedValue<i32>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            SpatialDifferencingDecodeIterator::FirstOrder(iter) => iter.next(),
            SpatialDifferencingDecodeIterator::SecondOrder(iter) => iter.next(),
        }
    }
}

pub(crate) struct FirstOrderSpatialDifferencingDecodeIterator<I, J> {
    iter: I,
    first_values: J,
    count: u32,
    prev: i32,
}

impl<I, J> FirstOrderSpatialDifferencingDecodeIterator<I, J> {
    pub(crate) fn new(iter: I, first_values: J) -> Self {
        Self {
            iter,
            first_values,
            count: 0,
            prev: 0,
        }
    }
}

impl<I, J> Iterator for FirstOrderSpatialDifferencingDecodeIterator<I, J>
where
    I: Iterator<Item = DecodedValue<i32>>,
    J: Iterator<Item = i32>,
{
    type Item = DecodedValue<i32>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            None => None,
            Some(Normal(v)) => match self.count {
                0 => {
                    self.prev = self.first_values.next().unwrap();
                    self.count += 1;
                    Some(Normal(self.prev))
                }
                _ => {
                    let v = v + self.prev;
                    self.prev = v;
                    Some(Normal(v))
                }
            },
            Some(missing) => Some(missing),
        }
    }
}

pub(crate) struct SecondOrderSpatialDifferencingDecodeIterator<I, J> {
    iter: I,
    first_values: J,
    count: u32,
    prev1: i32,
    prev2: i32,
}

impl<I, J> SecondOrderSpatialDifferencingDecodeIterator<I, J> {
    pub(crate) fn new(iter: I, first_values: J) -> Self {
        Self {
            iter,
            first_values,
            count: 0,
            prev1: 0,
            prev2: 0,
        }
    }
}

impl<I, J> Iterator for SecondOrderSpatialDifferencingDecodeIterator<I, J>
where
    I: Iterator<Item = DecodedValue<i32>>,
    J: Iterator<Item = i32>,
{
    type Item = DecodedValue<i32>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            None => None,
            Some(Normal(v)) => match self.count {
                0 => {
                    self.prev2 = self.first_values.next().unwrap();
                    self.count += 1;
                    Some(Normal(self.prev2))
                }
                1 => {
                    self.prev1 = self.first_values.next().unwrap();
                    self.count += 1;
                    Some(Normal(self.prev1))
                }
                _ => {
                    let v = v + 2 * self.prev1 - self.prev2;
                    self.prev2 = self.prev1;
                    self.prev1 = v;
                    Some(Normal(v))
                }
            },
            Some(missing) => Some(missing),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::missing::DecodedValue::{Missing1, Missing2},
        *,
    };

    macro_rules! test_spdiff_minimum_value {
        ($(($name:ident, $num_octets:expr, $expected:expr),)*) => ($(
            #[test]
            fn $name() {
                let spdiff_param = SpatialDifferencingParam {
                    order: 2,
                    extra_desc_num_octets: $num_octets,
                };
                let octets = (0x00..0x10).collect::<Vec<_>>();
                let spdiff_params =
                    SpatialDifferencingExtraDescriptors::new(&spdiff_param, &octets).unwrap();
                let actual = spdiff_params.minimum();
                assert_eq!(actual, $expected);
            }
        )*);
    }

    test_spdiff_minimum_value! {
        (spdiff_minimum_value_when_num_octets_is_1, 1, 0x02),
        (spdiff_minimum_value_when_num_octets_is_2, 2, 0x04_05),
        (spdiff_minimum_value_when_num_octets_is_3, 3, 0x06_07_08),
        (spdiff_minimum_value_when_num_octets_is_4, 4, 0x08_09_0a_0b),
    }

    macro_rules! test_spdiff_first_values {
        ($(($name:ident, $num_octets:expr, $expected:expr),)*) => ($(
            #[test]
            fn $name() {
                let spdiff_param = SpatialDifferencingParam {
                    order: 2,
                    extra_desc_num_octets: $num_octets,
                };
                let octets = (0x00..0x10).collect::<Vec<_>>();
                let spdiff_params =
                    SpatialDifferencingExtraDescriptors::new(&spdiff_param, &octets).unwrap();
                let actual = spdiff_params.first_values().collect::<Vec<_>>();
                assert_eq!(actual, $expected);
            }
        )*);
    }

    test_spdiff_first_values! {
        (spdiff_first_values_when_num_octets_is_1, 1, vec![0x00, 0x01]),
        (spdiff_first_values_when_num_octets_is_2, 2, vec![0x00_01, 0x02_03]),
        (spdiff_first_values_when_num_octets_is_3, 3, vec![0x00_01_02, 0x03_04_05]),
        (spdiff_first_values_when_num_octets_is_4, 4, vec![0x00_01_02_03, 0x04_05_06_07]),
    }

    macro_rules! test_first_order_spatial_differencing_decoding {
        ($(($name:ident, $input:expr, $expected:expr),)*) => ($(
            #[test]
            fn $name() {
                let input = $input
                    .into_iter();
                let first_values = vec![100].into_iter();
                let iter = FirstOrderSpatialDifferencingDecodeIterator::new(input, first_values);
                assert_eq!(
                    iter.collect::<Vec<_>>(),
                    $expected
                );
            }
        )*);
    }

    test_first_order_spatial_differencing_decoding! {
        (
            first_order_spatial_differencing_decoding_consisting_of_normal_values,
            (0_u32..10).map(|n| Normal(n as i32 * (-1_i32).pow(n))),
            vec![
                Normal(100),
                Normal(99),
                Normal(101),
                Normal(98),
                Normal(102),
                Normal(97),
                Normal(103),
                Normal(96),
                Normal(104),
                Normal(95),
            ]
        ),
        (
            first_order_spatial_differencing_decoding_with_missing_values_in_first_values,
            vec![
                Missing1,
                Normal(0),
                Normal(-1),
                Normal(2),
                Normal(-3),
                Normal(4),
                Normal(-5),
                Normal(6),
                Normal(-7),
                Normal(8),
            ],
            vec![
                Missing1,
                Normal(100),
                Normal(99),
                Normal(101),
                Normal(98),
                Normal(102),
                Normal(97),
                Normal(103),
                Normal(96),
                Normal(104),
            ]
        ),
        (
            first_order_spatial_differencing_decoding_with_missing_values_in_non_first_values,
            vec![
                Normal(0),
                Normal(-1),
                Missing1,
                Normal(2),
                Normal(-3),
                Normal(4),
                Missing2,
                Normal(-5),
                Normal(6),
                Normal(-7),
            ],
            vec![
                Normal(100),
                Normal(99),
                Missing1,
                Normal(101),
                Normal(98),
                Normal(102),
                Missing2,
                Normal(97),
                Normal(103),
                Normal(96),
            ]
        ),
    }

    macro_rules! test_second_order_spatial_differencing_decoding {
        ($(($name:ident, $input:expr, $expected:expr),)*) => ($(
            #[test]
            fn $name() {
                let input = $input
                    .into_iter();
                let first_values = vec![100, 99].into_iter();
                let iter = SecondOrderSpatialDifferencingDecodeIterator::new(input, first_values);
                assert_eq!(
                    iter.collect::<Vec<_>>(),
                    $expected
                );
            }
        )*);
    }

    test_second_order_spatial_differencing_decoding! {
        (
            second_order_spatial_differencing_decoding_consisting_of_normal_values,
            (0_u32..10).map(|n| Normal(n as i32 * (-1_i32).pow(n))),
            vec![
                Normal(100),
                Normal(99),
                Normal(100),
                Normal(98),
                Normal(100),
                Normal(97),
                Normal(100),
                Normal(96),
                Normal(100),
                Normal(95),
            ]
        ),
        (
            second_order_spatial_differencing_decoding_with_missing_values_in_first_values,
            vec![
                Missing1,
                Missing2,
                Normal(0),
                Normal(-1),
                Normal(2),
                Normal(-3),
                Normal(4),
                Normal(-5),
                Normal(6),
                Normal(-7),
            ],
            vec![
                Missing1,
                Missing2,
                Normal(100),
                Normal(99),
                Normal(100),
                Normal(98),
                Normal(100),
                Normal(97),
                Normal(100),
                Normal(96),
            ]
        ),
        (
            second_order_spatial_differencing_decoding_with_missing_values_in_non_first_values,
            vec![
                Normal(0),
                Normal(-1),
                Missing1,
                Normal(2),
                Normal(-3),
                Normal(4),
                Missing2,
                Normal(-5),
                Normal(6),
                Normal(-7),
            ],
            vec![
                Normal(100),
                Normal(99),
                Missing1,
                Normal(100),
                Normal(98),
                Normal(100),
                Missing2,
                Normal(97),
                Normal(100),
                Normal(96),
            ]
        ),
    }
}
