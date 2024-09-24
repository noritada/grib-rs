pub mod codetables;
mod context;
pub mod cookbook;
mod datatypes;
mod decoder;
mod error;
mod grid;
mod helpers;
mod parser;
mod reader;
pub mod utils;

pub use crate::{
    codetables::Code::{self, Name, Num},
    context::{
        from_reader, from_slice, Grib2, SectionBody, SectionInfo, SubMessageSection, TemplateInfo,
    },
    datatypes::*,
    decoder::*,
    error::*,
    grid::{
        EarthShapeDefinition, GaussianGridDefinition, GridPointIndexIterator, GridPointIterator,
        LambertGridDefinition, LatLonGridDefinition, PolarStereographicGridDefinition,
        ProjectionCentreFlag, ScanningMode,
    },
    parser::*,
    reader::*,
};

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
