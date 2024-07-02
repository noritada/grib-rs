use crate::utils::read_as;

#[derive(Debug, PartialEq, Eq)]
pub struct EarthShapeDefinition {
    pub shape_of_the_earth: u8,
    pub scale_factor_of_radius_of_spherical_earth: u8,
    pub scaled_value_of_radius_of_spherical_earth: u32,
    pub scale_factor_of_earth_major_axis: u8,
    pub scaled_value_of_earth_major_axis: u32,
    pub scale_factor_of_earth_minor_axis: u8,
    pub scaled_value_of_earth_minor_axis: u32,
}

impl EarthShapeDefinition {
    pub fn radii(&self) -> Option<(f64, f64)> {
        let radii = match self.shape_of_the_earth {
            0 => (6367470.0, 6367470.0),
            1 => {
                let radius = f64::from(self.scaled_value_of_radius_of_spherical_earth)
                    * f64::powf(
                        10.,
                        f64::from(self.scale_factor_of_radius_of_spherical_earth),
                    );
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
        let major = f64::from(self.scaled_value_of_earth_major_axis)
            * f64::powf(10., f64::from(self.scale_factor_of_earth_major_axis));
        let minor = f64::from(self.scaled_value_of_earth_minor_axis)
            * f64::powf(10., f64::from(self.scale_factor_of_earth_minor_axis));
        (major, minor)
    }

    pub(crate) fn from_buf(buf: &[u8]) -> Self {
        let shape_of_the_earth = read_as!(u8, buf, 0);
        let scale_factor_of_radius_of_spherical_earth = read_as!(u8, buf, 1);
        let scaled_value_of_radius_of_spherical_earth = read_as!(u32, buf, 2);
        let scale_factor_of_earth_major_axis = read_as!(u8, buf, 6);
        let scaled_value_of_earth_major_axis = read_as!(u32, buf, 7);
        let scale_factor_of_earth_minor_axis = read_as!(u8, buf, 11);
        let scaled_value_of_earth_minor_axis = read_as!(u32, buf, 12);
        Self {
            shape_of_the_earth,
            scale_factor_of_radius_of_spherical_earth,
            scaled_value_of_radius_of_spherical_earth,
            scale_factor_of_earth_major_axis,
            scaled_value_of_earth_major_axis,
            scale_factor_of_earth_minor_axis,
            scaled_value_of_earth_minor_axis,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{BufReader, Read},
    };

    use super::*;

    fn ungz_as_bytes<P>(file_path: P) -> Result<Vec<u8>, std::io::Error>
    where
        P: AsRef<std::path::Path>,
    {
        let mut buf = Vec::new();

        let f = File::open(file_path)?;
        let f = BufReader::new(f);
        let mut f = flate2::read::GzDecoder::new(f);
        f.read_to_end(&mut buf)?;

        Ok(buf)
    }

    fn unxz_as_bytes<P>(file_path: P) -> Result<Vec<u8>, std::io::Error>
    where
        P: AsRef<std::path::Path>,
    {
        let mut buf = Vec::new();

        let f = File::open(file_path)?;
        let f = BufReader::new(f);
        let mut f = xz2::bufread::XzDecoder::new(f);
        f.read_to_end(&mut buf)?;

        Ok(buf)
    }

    fn cat_as_bytes(file_name: &str) -> Result<Vec<u8>, std::io::Error> {
        let mut buf = Vec::new();

        let f = File::open(file_name)?;
        let mut f = BufReader::new(f);
        f.read_to_end(&mut buf)?;

        Ok(buf)
    }

    #[test]
    fn radii_for_shape_1() -> Result<(), Box<dyn std::error::Error>> {
        let buf = unxz_as_bytes("testdata/ds.critfireo.bin.xz")?;
        let earth_actual = EarthShapeDefinition::from_buf(&buf[0x83..]);
        let earth_expected = EarthShapeDefinition {
            shape_of_the_earth: 1,
            scale_factor_of_radius_of_spherical_earth: 0,
            scaled_value_of_radius_of_spherical_earth: 6371200,
            scale_factor_of_earth_major_axis: 0,
            scaled_value_of_earth_major_axis: 0,
            scale_factor_of_earth_minor_axis: 0,
            scaled_value_of_earth_minor_axis: 0,
        };
        assert_eq!(earth_actual, earth_expected);
        assert_eq!(earth_actual.radii(), Some((6_371_200., 6_371_200.)));

        Ok(())
    }

    #[test]
    fn radii_for_shape_2() -> Result<(), Box<dyn std::error::Error>> {
        let buf = ungz_as_bytes(
            "testdata/MRMS_ReflectivityAtLowestAltitude_00.50_20230406-120039.grib2.gz",
        )?;
        let earth_actual = EarthShapeDefinition::from_buf(&buf[0x33..]);
        let earth_expected = EarthShapeDefinition {
            shape_of_the_earth: 2,
            scale_factor_of_radius_of_spherical_earth: 1,
            scaled_value_of_radius_of_spherical_earth: 6367470,
            scale_factor_of_earth_major_axis: 1,
            scaled_value_of_earth_major_axis: 6378160,
            scale_factor_of_earth_minor_axis: 1,
            scaled_value_of_earth_minor_axis: 6356775,
        };
        assert_eq!(earth_actual, earth_expected);
        assert_eq!(earth_actual.radii(), Some((6_378_160.0, 6_356_775.0)));

        Ok(())
    }

    #[test]
    fn radii_for_shape_4() -> Result<(), Box<dyn std::error::Error>> {
        let buf = cat_as_bytes(
            "testdata/Z__C_RJTD_20160822020000_NOWC_GPV_Ggis10km_Pphw10_FH0000-0100_grib2.bin",
        )?;
        let earth_actual = EarthShapeDefinition::from_buf(&buf[0x33..]);
        let earth_expected = EarthShapeDefinition {
            shape_of_the_earth: 4,
            scale_factor_of_radius_of_spherical_earth: 0xff,
            scaled_value_of_radius_of_spherical_earth: 0xffffffff,
            scale_factor_of_earth_major_axis: 1,
            scaled_value_of_earth_major_axis: 63781370,
            scale_factor_of_earth_minor_axis: 1,
            scaled_value_of_earth_minor_axis: 63567523,
        };
        assert_eq!(earth_actual, earth_expected);
        assert_eq!(earth_actual.radii(), Some((6_378_137.0, 6_356_752.314)));

        Ok(())
    }

    #[test]
    fn radii_for_shape_6() -> Result<(), Box<dyn std::error::Error>> {
        let buf = unxz_as_bytes("testdata/gdas.t12z.pgrb2.0p25.f000.0-10.xz")?;
        let earth_actual = EarthShapeDefinition::from_buf(&buf[0x33..]);
        let earth_expected = EarthShapeDefinition {
            shape_of_the_earth: 6,
            scale_factor_of_radius_of_spherical_earth: 0,
            scaled_value_of_radius_of_spherical_earth: 0,
            scale_factor_of_earth_major_axis: 0,
            scaled_value_of_earth_major_axis: 0,
            scale_factor_of_earth_minor_axis: 0,
            scaled_value_of_earth_minor_axis: 0,
        };
        assert_eq!(earth_actual, earth_expected);
        assert_eq!(earth_actual.radii(), Some((6_371_229.0, 6_371_229.0)));

        Ok(())
    }
}
