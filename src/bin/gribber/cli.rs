#[cfg(unix)]
use pager::Pager;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
#[cfg(unix)]
use which::which;

use grib::context::Grib2;
use grib::reader::SeekableGrib2Reader;

pub fn grib<P>(path: P) -> anyhow::Result<Grib2<SeekableGrib2Reader<BufReader<File>>>>
where
    P: AsRef<Path>,
{
    let f = File::open(&path)?;
    let f = BufReader::new(f);
    Ok(grib::from_reader(f)?)
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
