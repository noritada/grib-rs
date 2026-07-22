use crate::def::grib2::template::param_set::EarthShape;

impl EarthShape {
    pub fn radii(&self) -> Option<(f64, f64)> {
        let radii = match self.shape {
            0 => (6367470.0, 6367470.0),
            1 => {
                let radius = f64::from(self.spherical_earth_radius_scaled_value)
                    * f64::powf(10., f64::from(self.spherical_earth_radius_scale_factor));
                (radius, radius)
            }
            2 => (6378160.0, 6356775.0),
            3 => {
                let (major, minor) = self.radii_defined();
                (major * 1000., minor * 1000.)
            }
            4 => (6378137.0, 6356752.314),
            5 => (6378137.0, 6356752.3142), // WGS84
            6 => (6371229.0, 6371229.0),
            7 => self.radii_defined(),
            8 => (6371200.0, 6371200.0),
            9.. => return None,
        };
        Some(radii)
    }

    fn radii_defined(&self) -> (f64, f64) {
        let major = f64::from(self.major_axis_scaled_value)
            * f64::powf(10., f64::from(self.major_axis_scale_factor));
        let minor = f64::from(self.minor_axis_scaled_value)
            * f64::powf(10., f64::from(self.minor_axis_scale_factor));
        (major, minor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TryFromSlice, test_utils::decompress_to_vec};

    #[test]
    fn radii_for_shape_1() -> Result<(), Box<dyn std::error::Error>> {
        let buf = decompress_to_vec(crate::test_utils::data::grib2::NOAA_NDFD_CRITFIREO)?;
        let mut pos = 0x83;
        let earth_actual = EarthShape::try_from_slice(&buf, &mut pos)?;
        let earth_expected = EarthShape {
            shape: 1,
            spherical_earth_radius_scale_factor: 0,
            spherical_earth_radius_scaled_value: 6371200,
            major_axis_scale_factor: 0,
            major_axis_scaled_value: 0,
            minor_axis_scale_factor: 0,
            minor_axis_scaled_value: 0,
        };
        assert_eq!(earth_actual, earth_expected);
        assert_eq!(earth_actual.radii(), Some((6_371_200., 6_371_200.)));

        Ok(())
    }

    #[test]
    fn radii_for_shape_2() -> Result<(), Box<dyn std::error::Error>> {
        let buf = decompress_to_vec(crate::test_utils::data::grib2::NOAA_MRMS_REFLECTIVITY)?;
        let mut pos = 0x33;
        let earth_actual = EarthShape::try_from_slice(&buf, &mut pos)?;
        let earth_expected = EarthShape {
            shape: 2,
            spherical_earth_radius_scale_factor: 1,
            spherical_earth_radius_scaled_value: 6367470,
            major_axis_scale_factor: 1,
            major_axis_scaled_value: 6378160,
            minor_axis_scale_factor: 1,
            minor_axis_scaled_value: 6356775,
        };
        assert_eq!(earth_actual, earth_expected);
        assert_eq!(earth_actual.radii(), Some((6_378_160.0, 6_356_775.0)));

        Ok(())
    }

    #[test]
    fn radii_for_shape_4() -> Result<(), Box<dyn std::error::Error>> {
        let buf = decompress_to_vec(crate::test_utils::data::grib2::JMA_TORNADO_NOWCAST)?;
        let mut pos = 0x33;
        let earth_actual = EarthShape::try_from_slice(&buf, &mut pos)?;
        let earth_expected = EarthShape {
            shape: 4,
            spherical_earth_radius_scale_factor: 0xff,
            spherical_earth_radius_scaled_value: 0xffffffff,
            major_axis_scale_factor: 1,
            major_axis_scaled_value: 63781370,
            minor_axis_scale_factor: 1,
            minor_axis_scaled_value: 63567523,
        };
        assert_eq!(earth_actual, earth_expected);
        assert_eq!(earth_actual.radii(), Some((6_378_137.0, 6_356_752.314)));

        Ok(())
    }

    #[test]
    fn radii_for_shape_6() -> Result<(), Box<dyn std::error::Error>> {
        let buf = decompress_to_vec(crate::test_utils::data::grib2::NOAA_GDAS_0_10)?;
        let mut pos = 0x33;
        let earth_actual = EarthShape::try_from_slice(&buf, &mut pos)?;
        let earth_expected = EarthShape {
            shape: 6,
            spherical_earth_radius_scale_factor: 0,
            spherical_earth_radius_scaled_value: 0,
            major_axis_scale_factor: 0,
            major_axis_scaled_value: 0,
            minor_axis_scale_factor: 0,
            minor_axis_scaled_value: 0,
        };
        assert_eq!(earth_actual, earth_expected);
        assert_eq!(earth_actual.radii(), Some((6_371_229.0, 6_371_229.0)));

        Ok(())
    }
}
