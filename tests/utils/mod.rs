use std::fs::File;
use std::io::{self, BufReader, Read, Write};
use tempfile::NamedTempFile;
use xz2::bufread::XzDecoder;

pub(crate) fn jma_tornado_nowcast_file() -> Result<NamedTempFile, io::Error> {
    unxz_to_tempfile(
        "testdata/Z__C_RJTD_20160822020000_NOWC_GPV_Ggis10km_Pphw10_FH0000-0100_grib2.bin.xz",
    )
}

pub(crate) fn jma_kousa_file() -> Result<NamedTempFile, io::Error> {
    unxz_to_tempfile(
        "testdata/Z__C_RJTD_20170221120000_MSG_GPV_Gll0p5deg_Pys_B20170221120000_F2017022115-2017022212_grib2.bin.xz",
    )
}

fn unxz_to_tempfile(file_name: &str) -> Result<NamedTempFile, io::Error> {
    let mut buf = Vec::new();
    let mut out = NamedTempFile::new()?;

    let f = File::open(file_name)?;
    let f = BufReader::new(f);
    let mut f = XzDecoder::new(f);
    f.read_to_end(&mut buf)?;
    out.write_all(&buf)?;

    Ok(out)
}

pub(crate) fn too_small_file() -> Result<NamedTempFile, io::Error> {
    let mut out = NamedTempFile::new()?;
    out.write_all(b"foo")?;

    Ok(out)
}

pub(crate) fn non_grib_file() -> Result<NamedTempFile, io::Error> {
    let mut out = NamedTempFile::new()?;
    out.write_all(b"foo foo foo foo foo foo foo foo ")?;

    Ok(out)
}
