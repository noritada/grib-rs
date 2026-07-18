pub(crate) use self::image::ImageIntoIter;
use crate::DecodeError;

pub(crate) fn decode_j2k(bytes: &[u8]) -> Result<ImageIntoIter, DecodeError> {
    let stream = stream::Stream::from_bytes(bytes)?;
    let decoder = decoder::Decoder::new(stream)?;
    decoder.setup(decoder::DecodeParams::default())?;
    let image = decoder.read_header()?;
    decoder.decode(&image)?;

    image.try_into_iter()
}

mod decoder;
mod image;
mod stream;
