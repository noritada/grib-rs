#![cfg_attr(docsrs, feature(doc_cfg))]

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
mod time;
pub mod utils;

pub use crate::{
    codetables::Code::{self, Name, Num},
    context::*,
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
    time::*,
};

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
