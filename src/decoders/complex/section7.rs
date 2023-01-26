use crate::{decoders::DecodeError, error::GribError, utils::grib_int_from_bytes};

use super::ComplexPackingDecodeError;

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
        if spdiff_order != 2 {
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
        FirstValues::new(&self)
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
}
