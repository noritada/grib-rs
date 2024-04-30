use super::{
    missing::DecodedValue::{self, Normal},
    ComplexPackingDecodeError,
};
use crate::{decoder::DecodeError, error::GribError, utils::grib_int_from_bytes};

pub(crate) struct SpatialDifferencingExtraDescriptors<'a> {
    slice: &'a [u8],
    num_octets: usize,
}

impl<'a> SpatialDifferencingExtraDescriptors<'a> {
    pub(crate) fn new(
        parent_slice: &'a [u8],
        spdiff_order: u8,
        num_octets: u8,
    ) -> Result<Self, GribError> {
        if !(1..=2).contains(&spdiff_order) {
            return Err(GribError::DecodeError(
                DecodeError::ComplexPackingDecodeError(ComplexPackingDecodeError::NotSupported),
            ));
        }
        if num_octets == 0 || num_octets > 4 {
            return Err(GribError::DecodeError(
                DecodeError::ComplexPackingDecodeError(ComplexPackingDecodeError::NotSupported),
            ));
        }
        let num_octets = usize::from(num_octets);
        let byte_length = usize::from(spdiff_order + 1) * num_octets;

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

pub(crate) enum SpatialDiffDecodeIterator<I, J> {
    FirstOrder(SpatialDiff1stOrderDecodeIterator<I, J>),
    SecondOrder(SpatialDiff2ndOrderDecodeIterator<I, J>),
}

impl<I, J> Iterator for SpatialDiffDecodeIterator<I, J>
where
    I: Iterator<Item = DecodedValue<i32>>,
    J: Iterator<Item = i32>,
{
    type Item = DecodedValue<i32>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            SpatialDiffDecodeIterator::FirstOrder(iter) => iter.next(),
            SpatialDiffDecodeIterator::SecondOrder(iter) => iter.next(),
        }
    }
}

pub(crate) struct SpatialDiff1stOrderDecodeIterator<I, J> {
    iter: I,
    first_values: J,
    count: u32,
    prev: i32,
}

impl<I, J> SpatialDiff1stOrderDecodeIterator<I, J> {
    pub(crate) fn new(iter: I, first_values: J) -> Self {
        Self {
            iter,
            first_values,
            count: 0,
            prev: 0,
        }
    }
}

impl<I, J> Iterator for SpatialDiff1stOrderDecodeIterator<I, J>
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

pub(crate) struct SpatialDiff2ndOrderDecodeIterator<I, J> {
    iter: I,
    first_values: J,
    count: u32,
    prev1: i32,
    prev2: i32,
}

impl<I, J> SpatialDiff2ndOrderDecodeIterator<I, J> {
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

impl<I, J> Iterator for SpatialDiff2ndOrderDecodeIterator<I, J>
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
    use super::*;

    macro_rules! test_spdiff_minimum_value {
        ($(($name:ident, $num_octets:expr, $expected:expr),)*) => ($(
            #[test]
            fn $name() {
                let octets = (0x00..0x10).collect::<Vec<_>>();
                let spdiff_params =
                    SpatialDifferencingExtraDescriptors::new(&octets, 2, $num_octets).unwrap();
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
                let octets = (0x00..0x10).collect::<Vec<_>>();
                let spdiff_params =
                    SpatialDifferencingExtraDescriptors::new(&octets, 2, $num_octets).unwrap();
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

    macro_rules! test_spatial_diff_1st_order_decoding{
        ($(($name:ident, $input:expr, $expected:expr),)*) => ($(
            #[test]
            fn $name() {
                let input = $input
                    .into_iter();
                let first_values = vec![100].into_iter();
                let iter = SpatialDiff1stOrderDecodeIterator::new(input, first_values);
                assert_eq!(
                    iter.collect::<Vec<_>>(),
                    $expected
                );
            }
        )*);
    }

    test_spatial_diff_1st_order_decoding! {
        (
            spatial_diff_1st_order_decoding_consisting_of_normal_values,
            (0_u32..10)
                .map(|n| DecodedValue::Normal(n as i32 * (-1_i32).pow(n))),
            vec![
                DecodedValue::Normal(100),
                DecodedValue::Normal(99),
                DecodedValue::Normal(101),
                DecodedValue::Normal(98),
                DecodedValue::Normal(102),
                DecodedValue::Normal(97),
                DecodedValue::Normal(103),
                DecodedValue::Normal(96),
                DecodedValue::Normal(104),
                DecodedValue::Normal(95),
            ]
        ),
        (
            spatial_diff_1st_order_decoding_with_missing_values_in_first_values,
            vec![
                DecodedValue::Missing1,
                DecodedValue::Normal(0),
                DecodedValue::Normal(-1),
                DecodedValue::Normal(2),
                DecodedValue::Normal(-3),
                DecodedValue::Normal(4),
                DecodedValue::Normal(-5),
                DecodedValue::Normal(6),
                DecodedValue::Normal(-7),
                DecodedValue::Normal(8),
            ],
            vec![
                DecodedValue::Missing1,
                DecodedValue::Normal(100),
                DecodedValue::Normal(99),
                DecodedValue::Normal(101),
                DecodedValue::Normal(98),
                DecodedValue::Normal(102),
                DecodedValue::Normal(97),
                DecodedValue::Normal(103),
                DecodedValue::Normal(96),
                DecodedValue::Normal(104),
            ]
        ),
        (
            spatial_diff_1st_order_decoding_with_missing_values_in_non_first_values,
            vec![
                DecodedValue::Normal(0),
                DecodedValue::Normal(-1),
                DecodedValue::Missing1,
                DecodedValue::Normal(2),
                DecodedValue::Normal(-3),
                DecodedValue::Normal(4),
                DecodedValue::Missing2,
                DecodedValue::Normal(-5),
                DecodedValue::Normal(6),
                DecodedValue::Normal(-7),
            ],
            vec![
                DecodedValue::Normal(100),
                DecodedValue::Normal(99),
                DecodedValue::Missing1,
                DecodedValue::Normal(101),
                DecodedValue::Normal(98),
                DecodedValue::Normal(102),
                DecodedValue::Missing2,
                DecodedValue::Normal(97),
                DecodedValue::Normal(103),
                DecodedValue::Normal(96),
            ]
        ),
    }

    macro_rules! test_spatial_diff_2nd_order_decoding{
        ($(($name:ident, $input:expr, $expected:expr),)*) => ($(
            #[test]
            fn $name() {
                let input = $input
                    .into_iter();
                let first_values = vec![100, 99].into_iter();
                let iter = SpatialDiff2ndOrderDecodeIterator::new(input, first_values);
                assert_eq!(
                    iter.collect::<Vec<_>>(),
                    $expected
                );
            }
        )*);
    }

    test_spatial_diff_2nd_order_decoding! {
        (
            spatial_diff_2nd_order_decoding_consisting_of_normal_values,
            (0_u32..10)
                .map(|n| DecodedValue::Normal(n as i32 * (-1_i32).pow(n))),
            vec![
                DecodedValue::Normal(100),
                DecodedValue::Normal(99),
                DecodedValue::Normal(100),
                DecodedValue::Normal(98),
                DecodedValue::Normal(100),
                DecodedValue::Normal(97),
                DecodedValue::Normal(100),
                DecodedValue::Normal(96),
                DecodedValue::Normal(100),
                DecodedValue::Normal(95),
            ]
        ),
        (
            spatial_diff_2nd_order_decoding_with_missing_values_in_first_values,
            vec![
                DecodedValue::Missing1,
                DecodedValue::Missing2,
                DecodedValue::Normal(0),
                DecodedValue::Normal(-1),
                DecodedValue::Normal(2),
                DecodedValue::Normal(-3),
                DecodedValue::Normal(4),
                DecodedValue::Normal(-5),
                DecodedValue::Normal(6),
                DecodedValue::Normal(-7),
            ],
            vec![
                DecodedValue::Missing1,
                DecodedValue::Missing2,
                DecodedValue::Normal(100),
                DecodedValue::Normal(99),
                DecodedValue::Normal(100),
                DecodedValue::Normal(98),
                DecodedValue::Normal(100),
                DecodedValue::Normal(97),
                DecodedValue::Normal(100),
                DecodedValue::Normal(96),
            ]
        ),
        (
            spatial_diff_2nd_order_decoding_with_missing_values_in_non_first_values,
            vec![
                DecodedValue::Normal(0),
                DecodedValue::Normal(-1),
                DecodedValue::Missing1,
                DecodedValue::Normal(2),
                DecodedValue::Normal(-3),
                DecodedValue::Normal(4),
                DecodedValue::Missing2,
                DecodedValue::Normal(-5),
                DecodedValue::Normal(6),
                DecodedValue::Normal(-7),
            ],
            vec![
                DecodedValue::Normal(100),
                DecodedValue::Normal(99),
                DecodedValue::Missing1,
                DecodedValue::Normal(100),
                DecodedValue::Normal(98),
                DecodedValue::Normal(100),
                DecodedValue::Missing2,
                DecodedValue::Normal(97),
                DecodedValue::Normal(100),
                DecodedValue::Normal(96),
            ]
        ),
    }
}
