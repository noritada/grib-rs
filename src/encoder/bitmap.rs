use grib_template_helpers::WriteToBuffer;

#[derive(Debug)]
pub(crate) struct Bitmap {
    data: Vec<u8>,
    pos: usize,
    offset: usize,
    has_nan: bool,
}

impl Bitmap {
    pub(crate) fn for_values(values: &[f64]) -> Self {
        Self {
            data: vec![0; values.len().div_ceil(8)],
            pos: 0,
            offset: 7,
            has_nan: false,
        }
    }

    pub(crate) fn push(&mut self, value: bool) {
        // corresponding point value is nonnan
        if value {
            self.data[self.pos] |= 1 << self.offset;
        } else {
            self.has_nan = true;
        }

        if self.offset == 0 {
            self.offset = 7;
            self.pos += 1;
        } else {
            self.offset -= 1;
        }
    }

    pub(crate) fn has_nan(&self) -> bool {
        self.has_nan
    }
}

impl WriteToBuffer for Bitmap {
    fn write_to_buffer(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
        let len = self.num_bytes_required();
        if buf.len() < len {
            return Err("destination buffer is too small");
        }

        buf[0..len].copy_from_slice(&self.data);
        Ok(len)
    }

    fn num_bytes_required(&self) -> usize {
        self.data.len()
    }
}
