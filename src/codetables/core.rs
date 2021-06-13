use num_enum::{TryFromPrimitive, TryFromPrimitiveError};

#[derive(Debug, PartialEq, Eq)]
pub enum Code<Enum, N> {
    Name(Enum),
    Num(N),
}

impl<Enum, N> From<Result<Enum, TryFromPrimitiveError<Enum>>> for Code<Enum, N>
where
    Enum: TryFromPrimitive<Primitive = N>,
{
    fn from(result: Result<Enum, TryFromPrimitiveError<Enum>>) -> Self {
        match result {
            Ok(e) => Self::Name(e),
            Err(TryFromPrimitiveError { number: n }) => Self::Num(n),
        }
    }
}
