#![cfg(feature = "jpeg2000-unpack-with-hayro")]

use std::{
    fs::File,
    io::{BufReader, Read},
};

fn read_xz(path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let f = File::open(path)?;
    let f = BufReader::new(f);
    let mut f = xz2::bufread::XzDecoder::new(f);
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    Ok(buf)
}

#[test]
fn decodes_cmc_jpeg2000_values_within_one_ulp_of_wgrib2_fixture()
-> Result<(), Box<dyn std::error::Error>> {
    let f = File::open("testdata/CMC_glb_TMP_ISBL_1_latlon.24x.24_2021051800_P000.grib2")?;
    let grib2 = grib::from_reader(BufReader::new(f))?;
    let (_index, submessage) = grib2.iter().next().ok_or("GRIB file has no submessages")?;
    let decoder = grib::Grib2SubmessageDecoder::from(submessage)?;

    let actual = decoder.dispatch()?.collect::<Vec<_>>();
    let expected = read_xz("testdata/gen/cmc-glb-wgrib2-le.bin.xz")?
        .chunks_exact(4)
        .map(|bytes| f32::from_le_bytes(bytes.try_into().expect("four-byte float")))
        .collect::<Vec<_>>();

    assert_eq!(actual.len(), expected.len());
    for (index, (actual, expected)) in actual.into_iter().zip(expected).enumerate() {
        let max_error = f32::EPSILON * expected.abs().max(1.0);
        assert!(
            (actual - expected).abs() <= max_error,
            "decoded value differs at index {index}: {actual} != {expected}"
        );
    }
    Ok(())
}
