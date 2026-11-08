use crate::{TryFromSlice, TryFromSliceResult};

/// The number of octets occupied by a value in the encoded representation.
///
/// This is independent of Rust's memory layout. Implementations must describe
/// a constant encoded size; optional and variable-length values do not qualify.
pub trait FixedSize {
    const SIZE: usize;
}

macro_rules! impl_fixed_size {
    ($(($ty:ty, $size:expr $(,)?)),* $(,)?) => {$(
        impl FixedSize for $ty {
            const SIZE: usize = $size;
        }
    )*};
}

impl_fixed_size![
    (u8, 1),
    (i8, 1),
    (u16, 2),
    (i16, 2),
    (u32, 4),
    (i32, 4),
    (u64, 8),
    (i64, 8),
    (f32, 4),
    (f64, 8)
];

impl<const N: usize> FixedSize for [u8; N] {
    const SIZE: usize = N;
}

/// The offset of `F`, in octets from the beginning of an encoded struct.
pub trait FieldOffset<F> {
    const OFFSET: usize;
}

/// Reads a field without deserializing the containing struct.
///
/// The buffer must start at the beginning of the encoded struct. Only the
/// selected field is deserialized; preceding and following fields are not
/// validated. The offset must identify a representation readable by `F`.
pub trait ReadFixedField<F>: FieldOffset<F>
where
    F: TryFromSlice,
{
    fn read_fixed_field(buf: &[u8]) -> TryFromSliceResult<F> {
        let field = buf.get(Self::OFFSET..).ok_or("slice length is too short")?;
        F::try_from_slice(field, &mut 0)
    }
}

impl<T, F> ReadFixedField<F> for T
where
    T: FieldOffset<F>,
    F: TryFromSlice,
{
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Template;

    impl FieldOffset<u16> for Template {
        const OFFSET: usize = 2;
    }

    #[test]
    fn reads_only_selected_field() {
        assert_eq!(Template::read_fixed_field(&[255, 255, 1, 2]), Ok(258));
    }

    #[test]
    fn rejects_short_buffers() {
        for len in 0..4 {
            assert_eq!(
                Template::read_fixed_field(&[0; 4][..len]),
                Err("slice length is too short"),
            );
        }
    }
}
