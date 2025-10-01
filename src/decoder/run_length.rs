use crate::{
    Grib2GpvUnpack,
    decoder::{DecodeError, Grib2SubmessageDecoder, stream::NBitwiseIterator},
};

pub(crate) struct RunLength<'d>(
    pub(crate) &'d Grib2SubmessageDecoder,
    pub(crate) &'d super::param::RunLengthPackingTemplate,
);

impl<'d> Grib2GpvUnpack for RunLength<'d> {
    type Iter<'a>
        = std::vec::IntoIter<f32>
    where
        Self: 'a;

    fn iter<'a>(&'a self) -> Result<Self::Iter<'a>, DecodeError> {
        let Self(target, template) = self;

        let mut level_map = Vec::with_capacity(template.run_length.leval_values.len() + 1);
        level_map.push(f32::NAN);
        let factor = 10_f32.powi(-i32::from(template.run_length.num_digits));
        level_map.extend(
            template
                .run_length
                .leval_values
                .iter()
                .map(|val| f32::from(*val) * factor),
        );

        let decoded_levels = rleunpack(
            target.sect7_payload(),
            template.run_length.nbit,
            template.run_length.maxv,
            Some(target.num_points_encoded()),
        )?;

        let level_to_value = |level: &u16| -> Result<f32, DecodeError> {
            let index: usize = (*level).into();
            level_map
                .get(index)
                .copied()
                .ok_or(DecodeError::from(format!("invalid level value: {level}")))
        };

        let decoded: Result<Vec<_>, _> = (*decoded_levels).iter().map(level_to_value).collect();
        Ok(decoded?.into_iter())
    }
}

// Since maxv is represented as a 16-bit integer, values are 16 bits or less.
fn rleunpack(
    input: &[u8],
    nbit: u8,
    maxv: u16,
    expected_len: Option<usize>,
) -> Result<Box<[u16]>, DecodeError> {
    let mut out_buf = match expected_len {
        Some(sz) => Vec::with_capacity(sz),
        None => Vec::new(),
    };

    let rlbase = maxv + 1;
    let lngu: usize = ((1u16 << nbit) - rlbase).into();
    let mut cached = None;
    let mut exp: usize = 1;
    let iter = NBitwiseIterator::new(input, usize::from(nbit));

    for value in iter {
        let value = value as u16;
        if rlbase > value {
            out_buf.push(value);
            cached = Some(value);
            exp = 1;
        } else {
            let prev = cached.ok_or(DecodeError::from("invalid first value"))?;
            let length: usize = ((value - rlbase) as usize) * exp;
            out_buf.append(&mut vec![prev; length]);
            exp *= lngu;
        }
    }

    if let Some(len) = expected_len
        && len != out_buf.len()
    {
        return Err(DecodeError::LengthMismatch);
    }

    Ok(out_buf.into_boxed_slice())
}

#[cfg(test)]
mod tests {
    use super::*;

    // This test data is based on 4-bit numbers of sample data
    // provided in JMBSC's document available in the following URL:
    // http://www.jmbsc.or.jp/jp/online/joho-sample/Run-Length_Encoding.pdf
    #[test]
    fn decode_data_with_run_length_encoding() {
        let input: Vec<u8> = vec![3, 9, 12, 6, 4, 15, 2, 1, 0, 13, 12, 2, 3];
        let output: Vec<u16> = vec![
            3, 9, 9, 6, 4, 4, 4, 4, 4, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3,
        ];
        let input: Vec<u8> = input.iter().map(|n| n + 240).collect();
        let output: Vec<u16> = output.iter().map(|n| n + 240).collect();

        assert_eq!(
            rleunpack(&input, 8, 250, Some(21)),
            Ok(output.into_boxed_slice())
        );
    }

    #[test]
    fn decode_data_with_run_length_encoding_with_multibyte_length() {
        let input: Vec<u8> = vec![0x00, 0x14, 0x1c];
        let output: Vec<u16> = vec![0; 6065];

        assert_eq!(rleunpack(&input, 8, 3, None), Ok(output.into_boxed_slice()));
    }
}
