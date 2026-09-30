use grib::{def::grib2, encoder, encoder::WriteGrib2Message as _};

#[allow(unused)]
fn grib2_message_from_references(
    discipline: u8,
    ident: &grib2::Section1Payload,
    product: &[u8],
    lats: &[f64],
    lngs: &[f64],
    values: &[f64],
) -> Result<Vec<u8>, String> {
    let grid = LatLngGridDef::try_from((lats, lngs))?;
    if values.len() != grid.num_points() {
        return Err(format!(
            "inconsistent numbers of points: values = {}, grid = {}",
            values.len(),
            grid.num_points()
        ));
    }
    let encoded = encoder::GpvEncoder::new(
        std::borrow::Cow::Borrowed(values),
        encoder::EncodingMethod::ComplexPacking(
            encoder::SimplePackingStrategy::Decimal(1),
            encoder::ComplexPackingStrategy::LookAhead(4),
            encoder::SpatialDifferencingOption::None,
        ),
    );

    let sect3_payload = grid.section3();
    let message = encoder::SingleGrib2Message::new(
        discipline,
        ident,
        None::<&[u8]>,
        sect3_payload,
        product,
        encoded,
    );
    let mut buf = vec![0; message.num_octets()];
    let mut _pos = 0;
    message.write(&mut buf).map_err(|_e| "unexpected error")?;

    Ok(buf)
}

struct LatLngGridDef(encoder::LatLonGrid);

impl LatLngGridDef {
    fn section3(&self) -> grib2::Section3Payload {
        grib2::Section3Payload {
            grid_def_source: 0,
            num_points: self.num_points() as u32,
            num_point_list_octets: 0,
            point_list_interpretation: 0,
            template_num: 0,
            template: grib2::GridDefinitionTemplate::_3_0(grib2::template::Template3_0 {
                earth: grib2::template::param_set::EarthShape {
                    shape: 6,
                    spherical_earth_radius: grib2::template::param_set::ScaledValue {
                        scale_factor: 0xff,
                        scaled_value: 0xffffffff,
                    },
                    major_axis: grib2::template::param_set::ScaledValue {
                        scale_factor: 0xff,
                        scaled_value: 0xffffffff,
                    },
                    minor_axis: grib2::template::param_set::ScaledValue {
                        scale_factor: 0xff,
                        scaled_value: 0xffffffff,
                    },
                },
                lat_lon: grib2::template::param_set::LatLonGrid::from(&self.0),
            }),
        }
    }

    fn num_points(&self) -> usize {
        self.0.shape.0 * self.0.shape.1
    }
}

impl TryFrom<(&[f64], &[f64])> for LatLngGridDef {
    type Error = String;

    fn try_from(value: (&[f64], &[f64])) -> Result<Self, Self::Error> {
        let (lats, lngs) = value;
        let ni = lngs.len();
        let nj = lats.len();
        if ni == 0 || nj == 0 {
            return Err(format!("invalid shape: ({ni}, {nj})"));
        }

        Ok(Self(encoder::LatLonGrid {
            shape: (ni, nj),
            first_point: (lats[0], lngs[0]),
            last_point: (*lats.last().unwrap(), *lngs.last().unwrap()),
            i_consecutive: true,
        }))
    }
}
