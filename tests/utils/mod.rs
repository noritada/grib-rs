use std::fs::File;
use std::io::{self, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use xz2::bufread::XzDecoder;

pub(crate) fn jma_tornado_nowcast_file() -> Result<NamedTempFile, io::Error> {
    unxz_to_tempfile(
        testdata_dir()
            .join("Z__C_RJTD_20160822020000_NOWC_GPV_Ggis10km_Pphw10_FH0000-0100_grib2.bin.xz"),
    )
}

pub(crate) fn jma_kousa_file() -> Result<NamedTempFile, io::Error> {
    unxz_to_tempfile(
        testdata_dir()
            .join("Z__C_RJTD_20170221120000_MSG_GPV_Gll0p5deg_Pys_B20170221120000_F2017022115-2017022212_grib2.bin.xz"),
    )
}

fn unxz_to_tempfile(file_path: PathBuf) -> Result<NamedTempFile, io::Error> {
    let mut buf = Vec::new();
    let mut out = NamedTempFile::new()?;

    let f = File::open(&file_path)?;
    let f = BufReader::new(f);
    let mut f = XzDecoder::new(f);
    f.read_to_end(&mut buf)?;
    out.write_all(&buf)?;

    Ok(out)
}

pub(crate) fn tornado_nowcast_be_bin_bytes() -> Result<Vec<u8>, io::Error> {
    unxz_as_bytes(testdata_dir().join("gen").join("tornado-wgrib2-be.bin.xz"))
}

pub(crate) fn tornado_nowcast_le_bin_bytes() -> Result<Vec<u8>, io::Error> {
    unxz_as_bytes(testdata_dir().join("gen").join("tornado-wgrib2-le.bin.xz"))
}

pub(crate) fn kousa_be_bin_bytes() -> Result<Vec<u8>, io::Error> {
    unxz_as_bytes(testdata_dir().join("gen").join("kousa-wgrib2-be.bin.xz"))
}

pub(crate) fn kousa_le_bin_bytes() -> Result<Vec<u8>, io::Error> {
    unxz_as_bytes(testdata_dir().join("gen").join("kousa-wgrib2-le.bin.xz"))
}

fn unxz_as_bytes(file_path: PathBuf) -> Result<Vec<u8>, io::Error> {
    let mut buf = Vec::new();

    let f = File::open(&file_path)?;
    let f = BufReader::new(f);
    let mut f = XzDecoder::new(f);
    f.read_to_end(&mut buf)?;

    Ok(buf)
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

pub(crate) fn cat_as_bytes(file_name: &str) -> Result<Vec<u8>, io::Error> {
    let mut buf = Vec::new();

    let f = File::open(file_name)?;
    let mut f = BufReader::new(f);
    f.read_to_end(&mut buf)?;

    Ok(buf)
}

fn testdata_dir() -> &'static Path {
    Path::new("testdata")
}
