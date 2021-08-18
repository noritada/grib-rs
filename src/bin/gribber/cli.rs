#[cfg(unix)]
use pager::Pager;
use std::fmt::{self, Display, Formatter};
use std::fs::File;
use std::io::{BufReader, Error};
use std::num::ParseIntError;
use std::path::Path;
#[cfg(unix)]
use which::which;

use grib::context::Grib2;
use grib::error::*;
use grib::reader::SeekableGrib2Reader;

pub enum CliError {
    Grib(GribError),
    ParseNumber(ParseIntError),
    IO(Error, String),
}

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Grib(e) => write!(f, "{}", e),
            Self::ParseNumber(e) => write!(f, "{:#?}", e),
            Self::IO(e, path) => write!(f, "{}: {}", e, path),
        }
    }
}

impl From<GribError> for CliError {
    fn from(e: GribError) -> Self {
        Self::Grib(e)
    }
}

impl From<ParseIntError> for CliError {
    fn from(e: ParseIntError) -> Self {
        Self::ParseNumber(e)
    }
}

pub fn grib(file_name: &str) -> Result<Grib2<SeekableGrib2Reader<BufReader<File>>>, CliError> {
    let path = Path::new(file_name);
    let f = File::open(&path).map_err(|e| CliError::IO(e, path.display().to_string()))?;
    let f = BufReader::new(f);
    Ok(Grib2::<SeekableGrib2Reader<BufReader<File>>>::read_with_seekable(f)?)
}

#[cfg(unix)]
pub fn start_pager() {
    if which("less").is_ok() {
        Pager::with_pager("less -R").setup();
    } else {
        Pager::new().setup();
    }
}

#[cfg(not(unix))]
pub fn start_pager() {}
