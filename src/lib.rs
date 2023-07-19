pub mod codetables;
mod context;
pub mod cookbook;
mod datatypes;
#[cfg(not(target_arch = "wasm32"))]
mod decoders;
mod error;
mod grid;
mod parser;
mod reader;
mod utils;

#[cfg(not(target_arch = "wasm32"))]
pub use crate::decoders::*;
pub use crate::{
    codetables::Code::{self, Name, Num},
    context::*,
    datatypes::*,
    error::*,
    grid::*,
    parser::*,
    reader::*,
};
