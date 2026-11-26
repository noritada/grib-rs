use num::traits::AsPrimitive;

use self::DecodedValue::{Missing1, Missing2, Normal};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum DecodedValue<N> {
    Normal(N),
    Missing1,
    Missing2,
}

impl<N: AsPrimitive<f32>> AsPrimitive<f32> for DecodedValue<N> {
    fn as_(self) -> f32 {
        match self {
            Normal(value) => value.as_(),
            Missing1 | Missing2 => f32::NAN,
        }
    }
}
