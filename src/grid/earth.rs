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
    use std::{
        fs::File,
        io::{BufReader, Read},
    };

    use grib_template_helpers::TryFromSlice;

    use super::*;

    fn get_uncompressed<P>(file_path: P) -> Result<Vec<u8>, std::io::Error>
    where
        P: AsRef<std::path::Path>,
    {
        let mut buf = Vec::new();

        let f = File::open(&file_path)?;
        let mut f = BufReader::new(f);
        match file_path.as_ref().extension().map(|s| s.as_encoded_bytes()) {
            Some(b"gz") => {
                let mut f = flate2::read::GzDecoder::new(f);
                f.read_to_end(&mut buf)?;
            }
            Some(b"xz") => {
                let mut f = xz2::bufread::XzDecoder::new(f);
                f.read_to_end(&mut buf)?;
            }
            _ => {
                f.read_to_end(&mut buf)?;
            }
        };

        Ok(buf)
    }

    #[test]
    fn radii_for_shape_1() -> Result<(), Box<dyn std::error::Error>> {
        let buf = get_uncompressed("testdata/ds.critfireo.bin.xz")?;
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
        let buf = get_uncompressed(
            "testdata/MRMS_ReflectivityAtLowestAltitude_00.50_20230406-120039.grib2.gz",
        )?;
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
        let buf = get_uncompressed(
            "testdata/Z__C_RJTD_20160822020000_NOWC_GPV_Ggis10km_Pphw10_FH0000-0100_grib2.bin",
        )?;
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
        let buf = get_uncompressed("testdata/gdas.t12z.pgrb2.0p25.f000.0-10.xz")?;
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
