use libaec_sys::{AEC_FLUSH, AEC_OK, aec_decode, aec_decode_end, aec_decode_init, aec_stream};

pub(crate) struct Stream(aec_stream);

impl Stream {
    pub(crate) fn new(bits_per_sample: u32, block_size: u32, rsi: u32, flags: u32) -> Self {
        let mut raw: aec_stream = unsafe { std::mem::zeroed() };
        raw.bits_per_sample = bits_per_sample;
        raw.block_size = block_size;
        raw.rsi = rsi;
        raw.flags = flags;
        Self(raw)
    }

    pub(crate) fn decode(&mut self, input: &[u8], output: &mut [u8]) -> Result<(), &'static str> {
        self.0.next_in = input.as_ptr();
        self.0.avail_in = input.len();
        self.0.next_out = output.as_mut_ptr();
        self.0.avail_out = output.len();

        let result = unsafe { aec_decode_init(&mut self.0) };
        if result as u32 != AEC_OK {
            return Err("aec_decode_init() failed");
        }

        let result = unsafe { aec_decode(&mut self.0, AEC_FLUSH as i32) };
        if result as u32 != AEC_OK {
            return Err("aec_decode() failed");
        }

        let result = unsafe { aec_decode_end(&mut self.0) };
        if result as u32 != AEC_OK {
            return Err("aec_decode_end() failed");
        }

        Ok(())
    }
}
