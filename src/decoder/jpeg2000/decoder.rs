use std::ptr::NonNull;

use openjpeg_sys as opj;

use super::{image::Image, stream::Stream};
use crate::DecodeError;

pub(crate) struct Codec(pub(crate) NonNull<opj::opj_codec_t>);

impl Drop for Codec {
    fn drop(&mut self) {
        unsafe {
            opj::opj_destroy_codec(self.0.as_ptr());
        }
    }
}

impl Codec {
    pub(crate) fn j2k() -> Result<Self, DecodeError> {
        Self::new(opj::OPJ_CODEC_FORMAT::OPJ_CODEC_J2K)
    }

    pub(crate) fn new(format: opj::OPJ_CODEC_FORMAT) -> Result<Self, DecodeError> {
        NonNull::new(unsafe { opj::opj_create_decompress(format) })
            .map(Self)
            .ok_or(DecodeError::from("setup of the decoder codec failed"))
    }

    pub(crate) fn as_ptr(&self) -> *mut opj::opj_codec_t {
        self.0.as_ptr()
    }
}

pub(crate) struct DecodeParams(opj::opj_dparameters);

impl Default for DecodeParams {
    fn default() -> Self {
        let mut decode_params = unsafe { std::mem::zeroed::<opj::opj_dparameters>() };
        unsafe { opj::opj_set_default_decoder_parameters(&mut decode_params as *mut _) };
        Self(decode_params)
    }
}

impl DecodeParams {
    pub(crate) fn as_ptr(&mut self) -> &mut opj::opj_dparameters {
        &mut self.0
    }
}

// Codec is always assumed to be J2K.
pub(crate) struct Decoder {
    codec: Codec,
    stream: Stream,
}

impl Decoder {
    pub(crate) fn new(stream: Stream) -> Result<Self, DecodeError> {
        let codec = Codec::j2k()?;
        Ok(Self { codec, stream })
    }

    pub(crate) fn setup(&self, mut params: DecodeParams) -> Result<(), DecodeError> {
        if unsafe { opj::opj_setup_decoder(self.as_ptr(), params.as_ptr()) } != 1 {
            return Err(DecodeError::from("setup of openjpeg decoder failed"));
        }
        Ok(())
    }

    pub(crate) fn read_header(&self) -> Result<Image, DecodeError> {
        let mut img: *mut opj::opj_image_t = std::ptr::null_mut();
        let result = unsafe { opj::opj_read_header(self.stream.as_ptr(), self.as_ptr(), &mut img) };
        let img = Image::new(img)?;

        if result == 1 {
            Ok(img)
        } else {
            Err(DecodeError::from(
                "reading of JPEG 2000 image header failed",
            ))
        }
    }

    pub(crate) fn decode(&self, img: &Image) -> Result<(), DecodeError> {
        if unsafe { opj::opj_decode(self.as_ptr(), self.stream.as_ptr(), img.as_ptr()) } == 1 {
            Ok(())
        } else {
            Err(DecodeError::from("decoding of JPEG 2000 image failed"))
        }
    }

    pub(crate) fn as_ptr(&self) -> *mut opj::opj_codec_t {
        self.codec.as_ptr()
    }
}
