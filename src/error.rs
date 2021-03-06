use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;

use crate::decoders::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GribError {
    InternalDataError,
    ParseError(ParseError),
    ValidationError(ValidationError),
    DecodeError(DecodeError),
}

impl Error for GribError {
    fn description(&self) -> &str {
        "grib error"
    }
}

impl From<ParseError> for GribError {
    fn from(e: ParseError) -> Self {
        Self::ParseError(e)
    }
}

impl From<ValidationError> for GribError {
    fn from(e: ValidationError) -> Self {
        Self::ValidationError(e)
    }
}

impl From<DecodeError> for GribError {
    fn from(e: DecodeError) -> Self {
        Self::DecodeError(e)
    }
}

impl Display for GribError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::InternalDataError => write!(f, "Something unexpected happend"),
            Self::ParseError(e) => write!(f, "{}", e),
            Self::ValidationError(e) => write!(f, "{}", e),
            Self::DecodeError(e) => write!(f, "{:#?}", e),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParseError {
    ReadError(String),
    FileTypeCheckError(String),
    NotGRIB,
    GRIBVersionMismatch(u8),
    UnknownSectionNumber(u8),
    EndSectionMismatch,
}

impl Error for ParseError {
    fn description(&self) -> &str {
        "grib parse error"
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::ReadError(s) => write!(f, "Read error: {}", s),
            Self::FileTypeCheckError(s) => write!(f, "Error in checking file type: {}", s),
            Self::NotGRIB => write!(f, "Not GRIB data"),
            Self::GRIBVersionMismatch(i) => write!(f, "Not GRIB version 2: {}", i),
            Self::UnknownSectionNumber(s) => write!(f, "Unknown section number: {}", s),
            Self::EndSectionMismatch => write!(f, "Content of End Section is not valid"),
        }
    }
}

impl From<io::Error> for ParseError {
    fn from(e: io::Error) -> Self {
        Self::ReadError(e.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValidationError {
    GRIB2IterationSuddenlyFinished,
    NoGridDefinition(usize),
    GRIB2WrongIteration(usize),
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::GRIB2IterationSuddenlyFinished => write!(f, "GRIB2 file suddenly finished"),
            Self::NoGridDefinition(i) => write!(f, "Grid Definition Section not found at {}", i),
            Self::GRIB2WrongIteration(i) => write!(f, "GRIB2 sections wrongly ordered at {}", i),
        }
    }
}
