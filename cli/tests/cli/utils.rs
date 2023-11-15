use std::{
    convert::TryInto,
    fs::File,
    io::{self, BufReader, Read, Write},
    path::Path,
};

use tempfile::NamedTempFile;
use xz2::bufread::XzDecoder;

pub(crate) mod testdata;

fn cat_to_tempfile<P>(file_path: P) -> Result<NamedTempFile, io::Error>
where
    P: AsRef<Path>,
{
    let mut buf = Vec::new();
    let mut out = NamedTempFile::new()?;

    let f = File::open(file_path)?;
    let mut f = BufReader::new(f);
    f.read_to_end(&mut buf)?;
    out.write_all(&buf)?;

    Ok(out)
}

fn gzcat_to_tempfile<P>(file_path: P) -> Result<NamedTempFile, io::Error>
where
    P: AsRef<Path>,
{
    let mut buf = Vec::new();
    let mut out = NamedTempFile::new()?;

    let f = File::open(file_path)?;
    let f = BufReader::new(f);
    let mut f = flate2::read::GzDecoder::new(f);
    f.read_to_end(&mut buf)?;
    out.write_all(&buf)?;

    Ok(out)
}

fn xzcat_to_tempfile<P>(file_path: P) -> Result<NamedTempFile, io::Error>
where
    P: AsRef<Path>,
{
    let mut buf = Vec::new();
    let mut out = NamedTempFile::new()?;

    let f = File::open(file_path)?;
    let f = BufReader::new(f);
    let mut f = XzDecoder::new(f);
    f.read_to_end(&mut buf)?;
    out.write_all(&buf)?;

    Ok(out)
}

fn unxz_as_bytes<P>(file_path: P) -> Result<Vec<u8>, io::Error>
where
    P: AsRef<Path>,
{
    let mut buf = Vec::new();

    let f = File::open(file_path)?;
    let f = BufReader::new(f);
    let mut f = XzDecoder::new(f);
    f.read_to_end(&mut buf)?;

    Ok(buf)
}

pub(crate) fn cat_as_bytes(file_name: &str) -> Result<Vec<u8>, io::Error> {
    let mut buf = Vec::new();

    let f = File::open(file_name)?;
    let mut f = BufReader::new(f);
    f.read_to_end(&mut buf)?;

    Ok(buf)
}

pub(crate) fn encode_le_bytes_using_simple_packing(
    input: Vec<u8>,
    ref_val: f32,
    exp: i16,
    dig: i16,
) -> Vec<i32> {
    let encode = |value: f32| -> i32 {
        let dig_factor = 10_f32.powi(dig as i32);
        let diff = value * dig_factor - ref_val;
        let encoded = diff * 2_f32.powi(-exp as i32);
        encoded.round() as i32
    };

    input
        .chunks(4)
        .map(|quad| f32::from_le_bytes(quad.try_into().unwrap())) // should be safely unwrapped
        .map(encode)
        .collect::<Vec<_>>()
}
