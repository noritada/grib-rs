use std::{
    env,
    error::Error,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

fn main() -> Result<(), Box<dyn Error>> {
    // This example script checks if all messages/submessages in all given GRIB2
    // files can be decoded.
    //
    // This script is a practical example and does not include detailed explanatory
    // comments. If you are interested in how to write code to decode, please check
    // the example `decode_layers`.

    let fnames = env::args().skip(1);
    for fname in fnames {
        let path = PathBuf::from(&fname);
        if !path.is_file() {
            continue;
        }
        check_decoding_file(path)?;
    }

    Ok(())
}

fn check_decoding_file<P>(path: P) -> Result<(), Box<dyn Error>>
where
    P: AsRef<Path>,
{
    eprintln!("processing {}", path.as_ref().to_str().unwrap_or(""));

    let f = File::open(path)?;
    let f = BufReader::new(f);
    let grib = grib::from_reader(f)?;

    if grib.is_empty() {
        return Err("empty GRIB2 data".into());
    }

    let len = grib.len();
    for (index, (message_index, submessage)) in grib.iter().enumerate() {
        eprintln!(
            "  {}.{} ({}/{})",
            &message_index.0, &message_index.1, index, len
        );
        let decoder = grib::Grib2SubmessageDecoder::from(submessage)?;
        let _values = decoder.dispatch()?.collect::<Vec<_>>();
    }

    Ok(())
}
