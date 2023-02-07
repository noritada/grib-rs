pub mod codetables;
pub mod context;
pub mod cookbook;
pub mod datatypes;
pub mod decoders;
pub mod error;
mod grid;
pub mod parser;
pub mod reader;
mod utils;

pub use crate::{
    context::{from_reader, from_slice, Grib2},
    grid::*,
};
