#[cfg(unix)]
use pager::Pager;
use std::fmt::{self, Display, Formatter};
use std::fs::File;
use std::io::{BufReader, Error};
use std::num::ParseIntError;
use std::path::Path;

use grib::context::{Grib2, GribError};
use grib::reader::SeekableGrib2Reader;

pub enum CliError {
    GribError(GribError),
    ParseNumberError(ParseIntError),
    IOError(Error, String),
}

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::GribError(e) => write!(f, "{}", e),
            Self::ParseNumberError(e) => write!(f, "{:#?}", e),
            Self::IOError(e, path) => write!(f, "{}: {}", e, path),
        }
    }
}

impl From<GribError> for CliError {
    fn from(e: GribError) -> Self {
        Self::GribError(e)
    }
}

impl From<ParseIntError> for CliError {
    fn from(e: ParseIntError) -> Self {
        Self::ParseNumberError(e)
    }
}

pub fn grib(file_name: &str) -> Result<Grib2<SeekableGrib2Reader<BufReader<File>>>, CliError> {
    let path = Path::new(file_name);
    let f = File::open(&path).map_err(|e| CliError::IOError(e, path.display().to_string()))?;
    let f = BufReader::new(f);
    Ok(Grib2::<SeekableGrib2Reader<BufReader<File>>>::read_with_seekable(f)?)
}

#[cfg(unix)]
pub fn start_pager() {
    Pager::new().setup();
}

#[cfg(not(unix))]
pub fn start_pager() {}
