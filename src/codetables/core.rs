use num_enum::{TryFromPrimitive, TryFromPrimitiveError};

#[derive(Debug, PartialEq, Eq)]
pub enum TableLookupResult<Enum, N> {
    Found(Enum),
    NotFound(N),
}

impl<Enum, N> From<Result<Enum, TryFromPrimitiveError<Enum>>> for TableLookupResult<Enum, N>
where
    Enum: TryFromPrimitive<Primitive = N>,
{
    fn from(result: Result<Enum, TryFromPrimitiveError<Enum>>) -> Self {
        match result {
            Ok(e) => Self::Found(e),
            Err(TryFromPrimitiveError { number: n }) => Self::NotFound(n),
        }
    }
}
