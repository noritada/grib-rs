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
    use super::*;

    #[test]
    fn radii_for_shape_1() {
        // testdata/ds.critfireo.bin.xz
        let earth = EarthShapeDefinition {
            shape_of_the_earth: 1,
            scale_factor_of_radius_of_spherical_earth: 0,
            scaled_value_of_radius_of_spherical_earth: 6371200,
            scale_factor_of_earth_major_axis: 0,
            scaled_value_of_earth_major_axis: 0,
            scale_factor_of_earth_minor_axis: 0,
            scaled_value_of_earth_minor_axis: 0,
        };
        assert_eq!(earth.radii(), Some((6_371_200., 6_371_200.)));
    }
}
