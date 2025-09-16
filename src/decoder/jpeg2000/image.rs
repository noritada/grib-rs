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

    #[cfg(feature = "jpeg2000-unpack-with-openjpeg-experimental")]
    pub(crate) fn try_into_iter(self) -> Result<ImageIntoIter, DecodeError> {
        let slice = if let [comp_gray] = self.components() {
            comp_gray.data()
        } else {
            return Err(DecodeError::from(
                "unexpected non-gray-scale image components",
            ));
        };
        let data = slice.as_ptr();
        let len = slice.len();

        Ok(ImageIntoIter {
            inner: self,
            data,
            len,
            index: 0,
        })
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

#[cfg(feature = "jpeg2000-unpack-with-openjpeg-experimental")]
pub(crate) struct ImageIntoIter {
    #[allow(dead_code)]
    inner: Image,
    data: *const i32,
    len: usize,
    index: usize,
}

#[cfg(feature = "jpeg2000-unpack-with-openjpeg-experimental")]
impl Iterator for ImageIntoIter {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            let value = unsafe { *self.data.add(self.index) };
            self.index += 1;
            Some(value)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len - self.index;
        (remaining, Some(remaining))
    }
}
