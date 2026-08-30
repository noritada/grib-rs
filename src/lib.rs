#![doc = include_str!(concat!(env!("OUT_DIR"), "/doc.txt"))]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod codetables;
mod context;
pub mod cookbook;
mod datatypes;
mod decoder;
pub mod def;
pub mod encoder;
mod error;
mod grid;
mod helpers;
mod parser;
mod projection;
mod reader;
#[cfg(test)]
mod test_utils;
mod time;
pub mod utils;
mod value;

pub use grib_template_helpers::{Dump, TryFromSlice, WriteToBuffer};

pub use crate::{
    codetables::Code::{self, Name, Num},
    context::*,
    datatypes::*,
    decoder::*,
    error::*,
    grid::{
        GridDefinitionTemplateValues, GridPointIndex, GridPointIndexIterator, GridPointLatLons,
        GridShortName, LatLons,
    },
    parser::*,
    projection::*,
    reader::*,
    time::*,
    value::*,
};

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
