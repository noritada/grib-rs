use std::{
    fs::File,
    io::{self, BufReader, Read, Write},
    path::Path,
};

use tempfile::NamedTempFile;

pub(crate) mod testdata;

fn write_uncompressed_to_tempfile<P>(file_path: P) -> Result<NamedTempFile, io::Error>
where
    P: AsRef<Path>,
{
    let mut out = NamedTempFile::new()?;
    let buf = get_uncompressed(file_path)?;
    out.write_all(&buf)?;
    Ok(out)
}

pub(crate) fn get_uncompressed<P>(file_path: P) -> Result<Vec<u8>, io::Error>
where
    P: AsRef<Path>,
{
    let mut buf = Vec::new();

    let f = File::open(&file_path)?;
    let mut f = BufReader::new(f);
    match file_path.as_ref().extension().map(|s| s.as_encoded_bytes()) {
        Some(b"gz") => {
            let mut f = flate2::read::GzDecoder::new(f);
            f.read_to_end(&mut buf)?;
        }
        Some(b"xz") => {
            let mut f = xz2::bufread::XzDecoder::new(f);
            f.read_to_end(&mut buf)?;
        }
        _ => {
            f.read_to_end(&mut buf)?;
        }
    };

    Ok(buf)
}
