#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum RunLengthEncodingUnpackError {
    NotSupported,
    InvalidFirstValue,
    LengthMismatch,
}

fn rleunpack(
    input: &[u8],
    nbit: u8,
    maxv: u8,
    expected_len: Option<usize>,
) -> Result<Box<[u8]>, RunLengthEncodingUnpackError> {
    if nbit != 8 {
        return Err(RunLengthEncodingUnpackError::NotSupported);
    }

    let mut out_buf = match expected_len {
        Some(sz) => Vec::with_capacity(sz),
        None => Vec::new(),
    };

    let rlbase = maxv + 1;
    let lngu = (2u16.pow(nbit.into()) - (rlbase as u16)) as usize;
    let mut cached = None;
    let mut exp: usize = 1;

    for value in input.iter() {
        let value = *value;

        if value < rlbase {
            out_buf.push(value);
            cached = Some(value);
            exp = 1;
        } else {
            let prev = cached.ok_or(RunLengthEncodingUnpackError::InvalidFirstValue)?;
            let length = ((value - rlbase) as usize) * exp;
            out_buf.append(&mut vec![prev; length as usize]);
            exp *= lngu;
        }
    }

    if let Some(len) = expected_len {
        if len != out_buf.len() {
            return Err(RunLengthEncodingUnpackError::LengthMismatch);
        }
    }

    Ok(out_buf.into_boxed_slice())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rleunpack_u8() {
        let input: Vec<u8> = vec![3, 9, 12, 6, 4, 15, 2, 1, 0, 13, 12, 2, 3];
        let output: Vec<u8> = vec![
            3, 9, 9, 6, 4, 4, 4, 4, 4, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3,
        ];
        let input: Vec<u8> = input.iter().map(|n| n + 240).collect();
        let output: Vec<u8> = output.iter().map(|n| n + 240).collect();

        assert_eq!(
            rleunpack(&input, 8, 250, Some(21)),
            Ok(output.into_boxed_slice())
        );
    }

    #[test]
    fn rleunpack_u8_long_length() {
        let input: Vec<u8> = vec![0x00, 0x14, 0x1c];
        let output: Vec<u8> = vec![0; 6065];

        assert_eq!(rleunpack(&input, 8, 3, None), Ok(output.into_boxed_slice()));
    }
}
