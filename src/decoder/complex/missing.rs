use num::ToPrimitive;

use self::DecodedValue::{Missing1, Missing2, Normal};

#[derive(Clone)]
pub(crate) enum DecodedValue<N: ToPrimitive> {
    Normal(N),
    Missing1,
    Missing2,
}

impl<N: ToPrimitive> ToPrimitive for DecodedValue<N> {
    fn to_i64(&self) -> Option<i64> {
        match self {
            Normal(v) => v.to_i64(),
            Missing1 => None,
            Missing2 => None,
        }
    }

    fn to_u64(&self) -> Option<u64> {
        match self {
            Normal(v) => v.to_u64(),
            Missing1 => None,
            Missing2 => None,
        }
    }

    fn to_f32(&self) -> Option<f32> {
        match self {
            Normal(v) => v.to_f32(),
            Missing1 => Some(f32::NAN),
            Missing2 => Some(f32::NAN),
        }
    }

    fn to_f64(&self) -> Option<f64> {
        match self {
            Normal(v) => v.to_f64(),
            Missing1 => Some(f64::NAN),
            Missing2 => Some(f64::NAN),
        }
    }
}
