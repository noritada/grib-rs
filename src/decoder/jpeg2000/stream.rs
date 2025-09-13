use std::ffi::c_void;

use openjpeg_sys as opj;

use crate::DecodeError;

struct Slice<'a> {
    offset: usize,
    buf: &'a [u8],
}

impl<'a> Slice<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self { offset: 0, buf }
    }

    fn remaining(&self) -> usize {
        self.buf.len() - self.offset
    }

    fn seek(&mut self, new_offset: usize) -> usize {
        self.offset = self.buf.len().min(new_offset);
        self.offset
    }

    fn consume(&mut self, nb_bytes: usize) -> usize {
        let new_offset = self.offset.saturating_add(nb_bytes);
        self.seek(new_offset)
    }

    fn read_into(&mut self, out_buffer: &mut [u8]) -> Option<usize> {
        let remaining = self.remaining();
        if remaining == 0 {
            return None;
        }

        let nb_bytes_read = std::cmp::min(remaining, out_buffer.len());
        let offset = self.offset;
        let end_off = self.consume(nb_bytes_read);
        out_buffer[0..nb_bytes_read].copy_from_slice(&self.buf[offset..end_off]);

        Some(nb_bytes_read)
    }
}

extern "C" fn buf_read_stream_read_fn(
    p_buffer: *mut c_void,
    nb_bytes: usize,
    p_data: *mut c_void,
) -> usize {
    if p_buffer.is_null() || nb_bytes == 0 {
        return usize::MAX;
    }

    let slice = unsafe { &mut *(p_data as *mut Slice) };
    let out_buf = unsafe { std::slice::from_raw_parts_mut(p_buffer as *mut u8, nb_bytes) };
    slice.read_into(out_buf).unwrap_or(usize::MAX)
}

extern "C" fn buf_read_stream_skip_fn(nb_bytes: i64, p_data: *mut c_void) -> i64 {
    let slice = unsafe { &mut *(p_data as *mut Slice) };
    slice.consume(nb_bytes as usize) as i64
}

extern "C" fn buf_read_stream_seek_fn(nb_bytes: i64, p_data: *mut c_void) -> i32 {
    let slice = unsafe { &mut *(p_data as *mut Slice) };
    let seek_offset = nb_bytes as usize;
    let new_offset = slice.seek(seek_offset);

    i32::from(seek_offset == new_offset)
}

extern "C" fn buf_read_stream_free_fn(p_data: *mut c_void) {
    let ptr = p_data as *mut Slice;
    drop(unsafe { Box::from_raw(ptr) })
}

pub(crate) struct Stream(*mut opj::opj_stream_t);

impl<'a> Drop for Stream {
    fn drop(&mut self) {
        unsafe {
            opj::opj_stream_destroy(self.0);
        }
    }
}

impl Stream {
    pub(crate) fn from_bytes(buf: &[u8]) -> Result<Self, DecodeError> {
        let len = buf.len();
        let data = Box::new(Slice::new(buf));

        unsafe {
            let stream = opj::opj_stream_default_create(1);
            opj::opj_stream_set_read_function(stream, Some(buf_read_stream_read_fn));
            opj::opj_stream_set_skip_function(stream, Some(buf_read_stream_skip_fn));
            opj::opj_stream_set_seek_function(stream, Some(buf_read_stream_seek_fn));
            opj::opj_stream_set_user_data_length(stream, len as u64);
            opj::opj_stream_set_user_data(
                stream,
                Box::into_raw(data) as *mut c_void,
                Some(buf_read_stream_free_fn),
            );

            Ok(Self(stream))
        }
    }

    pub(crate) fn as_ptr(&self) -> *mut opj::opj_stream_t {
        self.0
    }
}
