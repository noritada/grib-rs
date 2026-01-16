use std::{
    error,
    ffi::{CString, c_char},
    fmt,
    io::Read,
    path::Path,
    ptr::{self, NonNull},
};

use eccodes_sys::grib_handle;

#[derive(Debug)]
pub(crate) struct Grib2EncoderInput<'a> {
    pub vals: &'a [f64],
    pub latlon_shape: (usize, usize),
    pub first_point_latlon: (f64, f64),
    pub last_point_latlon: (f64, f64),
    pub date_time: grib::UtcDateTime,
}

pub(crate) fn encode_grib2(
    input: &Grib2EncoderInput<'_>,
    packing_type: &str,
) -> Result<Vec<u8>, Error> {
    let out = tempfile::NamedTempFile::new().map_err(|e| Error(e.to_string()))?;
    let out_fname = out.path();
    write_grib2(input, packing_type, out_fname)?;
    let mut f = out.reopen().map_err(|e| Error(e.to_string()))?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).map_err(|e| Error(e.to_string()))?;
    out.close().map_err(|e| Error(e.to_string()))?;

    Ok(buf)
}

pub(crate) fn write_grib2<P: AsRef<Path>>(
    input: &Grib2EncoderInput<'_>,
    packing_type: &str,
    out_fname: P,
) -> Result<(), Error> {
    let gh = GribHandle::new_from_samples("regular_ll_pl_grib2")?;

    // Section 0
    gh.set_long("discipline", 0)?;

    // Section 1
    gh.set_missing("centre")?;
    gh.set_long("subCentre", 0)?; // defined by originating centre, which is missing
    gh.set_long("tablesVersion", 29)?;
    gh.set_long("localTablesVersion", 0)?;
    gh.set_long("significanceOfReferenceTime", 0)?;
    gh.set_long("year", input.date_time.year as i64)?;
    gh.set_long("month", input.date_time.month as i64)?;
    gh.set_long("day", input.date_time.day as i64)?;
    gh.set_long("hour", input.date_time.hour as i64)?;
    gh.set_long("minute", input.date_time.minute as i64)?;
    gh.set_long("second", input.date_time.second as i64)?;
    gh.set_long("productionStatusOfProcessedData", 0)?; // Operational products
    gh.set_long("typeOfProcessedData", 0)?; // Analysis products

    // Section 3
    let (lat_num, lon_num) = input.latlon_shape;
    gh.set_long("Ni", lon_num as i64)?;
    gh.set_long("Nj", lat_num as i64)?;

    let (first_lat, first_lon) = input.first_point_latlon;
    gh.set_long("latitudeOfFirstGridPoint", (first_lat * 1e6) as i64)?;
    gh.set_long(
        "longitudeOfFirstGridPoint",
        (normalize_longitude(first_lon) * 1e6) as i64,
    )?;

    let (last_lat, last_lon) = input.last_point_latlon;
    gh.set_long("latitudeOfLastGridPoint", (last_lat * 1e6) as i64)?;
    gh.set_long(
        "longitudeOfLastGridPoint",
        (normalize_longitude(last_lon) * 1e6) as i64,
    )?;

    let i_inc = ((last_lon - first_lon) * 1e6).abs() as i64 / (lon_num - 1) as i64;
    gh.set_long("iDirectionIncrement", i_inc)?;
    let j_inc = ((last_lat - first_lat) * 1e6).abs() as i64 / (lat_num - 1) as i64;
    gh.set_long("jDirectionIncrement", j_inc)?;

    let mut scan_mode = 0b00000000;
    if first_lat < last_lat {
        scan_mode |= 0b01000000;
    }
    if first_lon > last_lon {
        scan_mode |= 0b10000000;
    }
    gh.set_long("scanningMode", scan_mode)?;

    // Section 4
    gh.set_long("productDefinitionTemplateNumber", 0)?;
    gh.set_long("parameterCategory", 15)?;
    gh.set_long("parameterNumber", 1)?;
    gh.set_long("typeOfGeneratingProcess", 0)?;
    gh.set_long("backgroundProcess", 0)?; // defined by originating centre, which is missing
    gh.set_long("generatingProcessIdentifier", 0)?; // defined by originating centre, which is missing
    gh.set_long("hoursAfterDataCutoff", 0)?;
    gh.set_long("minutesAfterDataCutoff", 0)?;
    gh.set_long("indicatorOfUnitForForecastTime", 1)?;
    gh.set_long("forecastTime", 0)?;
    gh.set_missing("typeOfFirstFixedSurface")?;
    gh.set_missing("scaleFactorOfFirstFixedSurface")?;
    gh.set_missing("scaledValueOfFirstFixedSurface")?;
    gh.set_missing("typeOfSecondFixedSurface")?;
    gh.set_missing("scaleFactorOfSecondFixedSurface")?;
    gh.set_missing("scaledValueOfSecondFixedSurface")?;

    // Sections 5 & 7
    if input.vals.iter().any(|val| val.is_nan()) {
        let vals = input
            .vals
            .iter()
            .map(|val| if val.is_nan() { -9999. } else { *val })
            .collect::<Vec<_>>();

        gh.set_double_array("values", &vals)?;
    } else {
        gh.set_double_array("values", input.vals)?;
    }

    gh.set_string("packingType", packing_type)?;

    gh.write_message(out_fname)?;

    Ok(())
}

struct GribHandle(NonNull<grib_handle>);

impl Drop for GribHandle {
    fn drop(&mut self) {
        unsafe { eccodes_sys::grib_handle_delete(self.as_ptr()) };
    }
}

impl GribHandle {
    fn new_from_samples(sample_name: &str) -> Result<Self, Error> {
        let gh = unsafe {
            eccodes_sys::grib_handle_new_from_samples(ptr::null_mut(), const_c_char(sample_name)?)
        };
        NonNull::new(gh)
            .map(Self)
            .ok_or(Error::from("setup of the GRIB2 writer failed"))
    }

    fn set_long(&self, key: &str, val: i64) -> Result<(), Error> {
        if unsafe { eccodes_sys::grib_set_long(self.as_ptr(), const_c_char(key)?, val) } != 0 {
            return Err(Error(format!(
                "setting a long value {} to `{}` failed",
                val, key
            )));
        }

        Ok(())
    }

    fn set_string(&self, key: &str, val: &str) -> Result<(), Error> {
        if unsafe {
            eccodes_sys::grib_set_string(
                self.as_ptr(),
                const_c_char(key)?,
                const_c_char(val)?,
                &mut val.len(),
            )
        } != 0
        {
            return Err(Error(format!(
                r#"setting a string value "{}" to `{}` failed"#,
                val, key,
            )));
        };

        Ok(())
    }

    fn set_double_array(&self, key: &str, val: &[f64]) -> Result<(), Error> {
        if unsafe {
            eccodes_sys::grib_set_double_array(
                self.as_ptr(),
                const_c_char(key)?,
                val.as_ptr(),
                val.len(),
            )
        } != 0
        {
            return Err(Error(format!("setting a double array to `{}` failed", key)));
        };

        Ok(())
    }

    fn set_missing(&self, key: &str) -> Result<(), Error> {
        if unsafe { eccodes_sys::codes_set_missing(self.as_ptr(), const_c_char(key)?) } != 0 {
            return Err(Error(format!(
                "setting a missing value to `{}` failed",
                key
            )));
        };

        Ok(())
    }

    fn write_message<P: AsRef<Path>>(&self, out_fname: P) -> Result<(), Error> {
        let out_fname = out_fname.as_ref().to_str().unwrap_or_default();
        if unsafe {
            eccodes_sys::grib_write_message(
                self.as_ptr(),
                const_c_char(out_fname)?,
                const_c_char("w")?,
            )
        } != 0
        {
            return Err(Error(format!(
                r#"writing GRIB2 message to "{}" failed"#,
                out_fname
            )));
        };

        Ok(())
    }

    fn as_ptr(&self) -> *mut grib_handle {
        self.0.as_ptr()
    }
}

fn const_c_char(s: &str) -> Result<*const c_char, Error> {
    let s = CString::new(s).map_err(|e| Error(e.to_string()))?;
    let s: *const c_char = s.into_raw() as *const c_char;
    Ok(s)
}

/// Normalize longitude values in preparation for GRIB2 encoding.
/// This is required since wgrib2 cannot handle negative longitudes:
///
/// ```text
/// > wgrib2 -V max_composite_complex.grib2
/// BAD GDS:lon1=2223.983648 lon2=2178.983648 should be 0..360
/// 1:0:vt=2007032312:1000 mb:anl:TMP Temperature [K]:
///     ndata=2991461:undef=0:mean=0.11207:min=0:max=62.3716
///     grid_template=0:winds(N/S):
///         lat-lon grid:(1661 x 1801) units 1e-06 input WE:NS output WE:SN res 48
///         lat 6.500000 to -35.000000 by 0.023055
///         lon 2223.983648 to 2178.983648 by 0.027108 #points=2991461
/// ```
fn normalize_longitude(val: f64) -> f64 {
    if val < 0. { val + 360. } else { val }
}

#[derive(Debug)]
pub(crate) struct Error(String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}
