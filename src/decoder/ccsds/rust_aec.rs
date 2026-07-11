use ::rust_aec::{AecParams, decode_into, flags_from_grib2_ccsds_flags};

pub(crate) struct Stream {
    bits_per_sample: u32,
    block_size: u32,
    rsi: u32,
    flags: u32,
    output_samples: Option<usize>,
}

impl Stream {
    pub(crate) fn new(bits_per_sample: u32, block_size: u32, rsi: u32, flags: u32) -> Self {
        Self {
            bits_per_sample,
            block_size,
            rsi,
            flags,
            output_samples: None,
        }
    }

    pub(crate) fn set_output_samples(&mut self, output_samples: usize) {
        self.output_samples = Some(output_samples);
    }

    pub(crate) fn decode(&mut self, input: &[u8], output: &mut [u8]) -> Result<(), String> {
        let bits_per_sample = u8::try_from(self.bits_per_sample)
            .map_err(|_| format!("bits_per_sample is too large: {}", self.bits_per_sample))?;
        let output_samples = self
            .output_samples
            .ok_or_else(|| "CCSDS output sample count was not set".to_owned())?;

        let params = AecParams::new(
            bits_per_sample,
            self.block_size,
            self.rsi,
            flags_from_grib2_ccsds_flags((self.flags & 0xff) as u8),
        );
        decode_into(input, params, output_samples, output)
            .map_err(|err| format!("rust-aec decode failed: {err}"))
    }
}
