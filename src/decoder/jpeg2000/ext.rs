/*!
This code is based on src/lib.rs from [jp2k](https://crates.io/crates/jp2k) (https://github.com/kardeiz/jp2k), version 0.3.1, which was previously forked from https://framagit.org/leoschwarz/jpeg2000-rust before its GPL-v3 relicensing.


## Original warnings and license statement

### Warning
Please be advised that using C code means this crate is likely vulnerable to various memory exploits, e.g. see [http://cve.mitre.org/cgi-bin/cvename.cgi?name=CVE-2016-8332](CVE-2016-8332) for an actual example from the past.

As soon as someone writes an efficient JPEG2000 decoder in pure Rust you should probably switch over to that.

### License
You can use the Rust code in the directories `src` and `openjp2-sys/src` under the terms of either the MIT license (`LICENSE-MIT` file) or the Apache license (`LICENSE-APACHE` file). Please note that this will link statically to OpenJPEG, which has its own license which you can find at `openjpeg-sys/libopenjpeg/LICENSE` (you might have to check out the git submodule first).
*/

use std::{
    os::raw::c_void,
    ptr::{self, NonNull},
};

use openjpeg_sys as opj;
use opj::OPJ_CODEC_FORMAT;

use crate::DecodeError;

pub(crate) struct Stream(pub(crate) *mut opj::opj_stream_t);

impl Drop for Stream {
    fn drop(&mut self) {
        unsafe {
            opj::opj_stream_destroy(self.0);
        }
    }
}

impl Stream {
    pub(crate) fn from_bytes(buf: &[u8]) -> Result<Self, DecodeError> {
        #[derive(Debug)]
        struct SliceWithOffset<'a> {
            buf: &'a [u8],
            offset: usize,
        }

        unsafe extern "C" fn opj_stream_free_user_data_fn(p_user_data: *mut c_void) {
            drop(Box::from_raw(p_user_data as *mut SliceWithOffset))
        }

        unsafe extern "C" fn opj_stream_read_fn(
            p_buffer: *mut c_void,
            p_nb_bytes: usize,
            p_user_data: *mut c_void,
        ) -> usize {
            if p_buffer.is_null() {
                return 0;
            }

            let user_data = p_user_data as *mut SliceWithOffset;

            #[allow(clippy::needless_borrow)]
            let len = (&(*user_data).buf).len();

            let offset = (*user_data).offset;

            let bytes_left = len - offset;

            let bytes_read = std::cmp::min(bytes_left, p_nb_bytes);

            #[allow(clippy::needless_borrow)]
            let slice = &(&(*user_data).buf)[offset..offset + bytes_read];

            std::ptr::copy_nonoverlapping(slice.as_ptr(), p_buffer as *mut u8, bytes_read);

            (*user_data).offset += bytes_read;

            bytes_read
        }

        let buf_len = buf.len();
        let user_data = Box::new(SliceWithOffset { buf, offset: 0 });

        let ptr = unsafe {
            let jp2_stream = opj::opj_stream_default_create(1);
            opj::opj_stream_set_read_function(jp2_stream, Some(opj_stream_read_fn));
            opj::opj_stream_set_user_data_length(jp2_stream, buf_len as u64);
            opj::opj_stream_set_user_data(
                jp2_stream,
                Box::into_raw(user_data) as *mut c_void,
                Some(opj_stream_free_user_data_fn),
            );
            jp2_stream
        };

        Ok(Self(ptr))
    }
}

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
        Self::create(OPJ_CODEC_FORMAT::OPJ_CODEC_J2K)
    }

    pub(crate) fn create(format: OPJ_CODEC_FORMAT) -> Result<Self, DecodeError> {
        NonNull::new(unsafe { opj::opj_create_decompress(format) })
            .map(Self)
            .ok_or(DecodeError::from("setup of j2k decoder failed"))
    }
}

#[derive(Debug)]
pub(crate) struct Image(pub(crate) *mut opj::opj_image_t);

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            opj::opj_image_destroy(self.0);
        }
    }
}

impl Image {
    pub(crate) fn new() -> Self {
        Image(ptr::null_mut())
    }

    pub(crate) fn num_components(&self) -> u32 {
        unsafe { (*self.0).numcomps }
    }

    pub(crate) fn components(&self) -> &[opj::opj_image_comp_t] {
        let comps_len = self.num_components();
        unsafe { std::slice::from_raw_parts((*self.0).comps, comps_len as usize) }
    }
}
