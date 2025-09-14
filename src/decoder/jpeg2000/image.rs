use std::ptr::NonNull;

use openjpeg_sys as opj;

use crate::DecodeError;

#[derive(Debug)]
pub(crate) struct Image(NonNull<opj::opj_image_t>);

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            opj::opj_image_destroy(self.0.as_ptr());
        }
    }
}

impl Image {
    pub(crate) fn new(ptr: *mut opj::opj_image_t) -> Result<Self, DecodeError> {
        let img = NonNull::new(ptr)
            .ok_or_else(|| DecodeError::from("initialization of the JPEG 2000 image failed"))?;
        Ok(Self(img))
    }

    fn inner(&self) -> &opj::opj_image_t {
        unsafe { &(*self.0.as_ptr()) }
    }

    pub(crate) fn components(&self) -> &[ImageComponent] {
        let img = self.inner();
        let numcomps = img.numcomps;
        unsafe { std::slice::from_raw_parts(img.comps as *mut ImageComponent, numcomps as usize) }
    }

    pub(crate) fn as_ptr(&self) -> *mut opj::opj_image_t {
        self.0.as_ptr()
    }
}

pub(crate) struct ImageComponent(opj::opj_image_comp_t);

impl ImageComponent {
    pub(crate) fn data(&self) -> &[i32] {
        let len = (self.0.w * self.0.h) as usize;
        unsafe { std::slice::from_raw_parts(self.0.data, len) }
    }
}
