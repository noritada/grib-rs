use std::cell::{Ref, RefCell};

pub use complex::*;
pub use grid::*;
pub use message::*;
pub use simple::*;

use crate::def::grib2::template::param_set;

pub struct Encoder<'d> {
    data: std::borrow::Cow<'d, [f64]>,
    method: EncodingMethod,
    encoded: RefCell<Option<EncodeOutput>>,
}

impl<'d> Encoder<'d> {
    pub fn new(data: std::borrow::Cow<'d, [f64]>, method: EncodingMethod) -> Self {
        Self {
            data,
            method,
            encoded: RefCell::new(None),
        }
    }

    pub fn get_encoded(&'_ self) -> Ref<'_, EncodeOutput> {
        if self.encoded.borrow().is_none() {
            *self.encoded.borrow_mut() = Some(self.encode());
        }

        Ref::map(self.encoded.borrow(), |c| c.as_ref().unwrap())
    }

    fn encode(&self) -> EncodeOutput {
        let output = match &self.method {
            EncodingMethod::SimplePacking(simple_packing_strategy) => {
                let encoder = simple::Encoder::new(&self.data, simple_packing_strategy.clone());
                EncodeOutputInner::SimplePacking(encoder.encode())
            }
            EncodingMethod::ComplexPacking(
                simple_packing_strategy,
                complex_packing_strategy,
                _spatial_differencing_option,
            ) => {
                let encoder = complex::Encoder::new(
                    &self.data,
                    simple_packing_strategy.clone(),
                    complex_packing_strategy.clone(),
                );
                EncodeOutputInner::ComplexPacking(encoder.encode())
            }
        };
        EncodeOutput(output)
    }
}

impl<'d> WriteGrib2PointValues for Encoder<'d> {
    fn data_sections_len(&self) -> usize {
        let encoded = self.get_encoded();
        encoded.section5_len() + encoded.section6_len() + encoded.section7_len()
    }

    fn write_data_sections(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
        let encoded = self.get_encoded();
        let mut pos = 0;
        pos += encoded.write_section5(&mut buf[pos..])?;
        pos += encoded.write_section6(&mut buf[pos..])?;
        pos += encoded.write_section7(&mut buf[pos..])?;
        Ok(pos)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[non_exhaustive]
pub enum EncodingMethod {
    /// Simple packing.
    SimplePacking(SimplePackingStrategy),
    /// Complex packing.
    ComplexPacking(
        SimplePackingStrategy,
        ComplexPackingStrategy,
        SpatialDifferencingOption,
    ),
}

/// Data obtained through encoding. Instances are typically used to write GRIB2
/// data via the methods defined in [`WriteGrib2DataSections`].
#[derive(Debug)]
pub struct EncodeOutput(EncodeOutputInner);

impl EncodeOutput {
    /// Returns the parameter set.
    pub fn params(&self) -> EncodeOutputParams<'_> {
        match &self.0 {
            EncodeOutputInner::SimplePacking(encoded) => {
                EncodeOutputParams::SimplePacking(encoded.params())
            }
            EncodeOutputInner::ComplexPacking(encoded) => {
                let (simple, complex) = encoded.params();
                EncodeOutputParams::ComplexPacking(simple, complex)
            }
        }
    }
}

impl WriteGrib2DataSections for EncodeOutput {
    fn section5_len(&self) -> usize {
        match &self.0 {
            EncodeOutputInner::SimplePacking(encoded) => encoded.section5_len(),
            EncodeOutputInner::ComplexPacking(encoded) => encoded.section5_len(),
        }
    }

    fn write_section5(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
        match &self.0 {
            EncodeOutputInner::SimplePacking(encoded) => encoded.write_section5(buf),
            EncodeOutputInner::ComplexPacking(encoded) => encoded.write_section5(buf),
        }
    }

    fn section6_len(&self) -> usize {
        match &self.0 {
            EncodeOutputInner::SimplePacking(encoded) => encoded.section6_len(),
            EncodeOutputInner::ComplexPacking(encoded) => encoded.section6_len(),
        }
    }

    fn write_section6(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
        match &self.0 {
            EncodeOutputInner::SimplePacking(encoded) => encoded.write_section6(buf),
            EncodeOutputInner::ComplexPacking(encoded) => encoded.write_section6(buf),
        }
    }

    fn section7_len(&self) -> usize {
        match &self.0 {
            EncodeOutputInner::SimplePacking(encoded) => encoded.section7_len(),
            EncodeOutputInner::ComplexPacking(encoded) => encoded.section7_len(),
        }
    }

    fn write_section7(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
        match &self.0 {
            EncodeOutputInner::SimplePacking(encoded) => encoded.write_section7(buf),
            EncodeOutputInner::ComplexPacking(encoded) => encoded.write_section7(buf),
        }
    }
}

#[non_exhaustive]
pub enum EncodeOutputParams<'a> {
    SimplePacking(&'a param_set::SimplePacking),
    ComplexPacking(&'a param_set::SimplePacking, &'a param_set::ComplexPacking),
}

#[derive(Debug)]
enum EncodeOutputInner {
    SimplePacking(simple::Encoded),
    ComplexPacking(complex::Encoded),
}

trait Encode {
    type Output;

    fn encode(&self) -> Self::Output;
}

mod bitmap;
mod complex;
mod grid;
mod helpers;
mod message;
mod simple;
mod writer;
