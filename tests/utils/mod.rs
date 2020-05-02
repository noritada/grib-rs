use std::fs::File;
use std::io::{self, BufReader, Read, Write};
use tempfile::NamedTempFile;
use xz2::bufread::XzDecoder;

pub(crate) fn jma_tornado_nowcast_file() -> Result<NamedTempFile, io::Error> {
    let mut buf = Vec::new();
    let mut out = NamedTempFile::new()?;

    let f = File::open(
        "testdata/Z__C_RJTD_20160822020000_NOWC_GPV_Ggis10km_Pphw10_FH0000-0100_grib2.bin.xz",
    )?;
    let f = BufReader::new(f);
    let mut f = XzDecoder::new(f);
    f.read_to_end(&mut buf)?;
    out.write_all(&buf)?;

    Ok(out)
}
