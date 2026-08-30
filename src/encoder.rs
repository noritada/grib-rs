//! GRIB2 data encoder.
//!
//! # The complexity of GRIB2
//!
//! GRIB2 is a data format specifically designed to store grid-point values
//! related to meteorological data. Therefore, it is easy to imagine that, in
//! addition to the values at each grid point, information such as the latitude
//! and longitude of each point, the date and time, and meteorological elements
//! must also be included as metadata. Furthermore, even for grid-point values
//! represented as arrays of real numbers, GRIB2 generally does not store
//! floating-point values as-is. Instead, the values are rounded based on their
//! precision and then compressed and stored as an array of integers--that is,
//! discrete numerical values.
//!
//! Consequently, unlike highly versatile formats such as NetCDF and HDF, which
//! allow for the flexible serialization of large amounts of numerical data, or
//! Gzip and Xz, which can freely compress arbitrary data, GRIB2 is a format
//! with specific constraints. On the other hand, GRIB2 is a format whose
//! specifications prevent situations where "a value has been obtained but its
//! meaning is unclear." It can also be described as a format capable of
//! "representing" meteorological data by incorporating not only raw numerical
//! values but also the complexities of the real world, such as missing values
//! and precision information.
//!
//! # Goal of this module
//!
//! The goal of this module is to enable users to create GRIB2 data--which
//! has many parameters and some configurable options within its
//! constraints--using an API that is as user-friendly as possible (although
//! some parts are currently not user-friendly).
//!
//! Internally, a single GRIB2 dataset (message) is composed of a collection of
//! blocks called "sections." However, this module's high-level API avoids using
//! the term "section" and instead focuses on "what information needs to be
//! configured" and "how you want to represent the meteorological data."
//!
//! # High-level and low-level APIs
//!
//! This module provides 3 structs: [`SingleGrib2Message`],
//! [`MultiGridGrib2Message`], and [`MultiProductGrib2Message`]. By providing
//! all the necessary information to these structs (which may be a bit of a
//! challenge), you can create GRIB2 messages. These structs constitute a
//! high-level API designed to accommodate the most common use cases.
//! If your use case does not fit these scenarios, you will need to implement
//! the low-level API [`WriteGrib2Message`] and its sub-traits:
//! [`WriteGrib2MessageIterL1`], [`WriteGrib2MessageIterL2`], and
//! [`WriteGrib2MessageIterL3`].
//!
//! # Generating data sets comprising multiple elements
//!
//! It is rare to have only one type of grid point value data for a given time
//! that you want to generate and provide. As shown below, in many cases, you
//! will likely want to generate and provide multiple types of grid point value
//! data together.
//!
//! - Different meteorological elements (temperature, pressure, humidity, wind
//!   speed in 2 directions)
//! - Different forecast times (1 hour later, 2 hours later, 3 hours later)
//! - Different altitude levels (1000 hPa, 925 hPa, 850 hPa)
//!
//! We will refer to this type of data here as "data sets comprising multiple
//! elements." The term "element" here is used in a general sense and is not
//! limited to meteorological elements.
//!
//! When representing data sets comprising multiple elements in GRIB2, there are
//! 2 main options, or 3 when broken down in detail:
//!
//! - A. Include them in a single message
//! - B. Create separate messages (but store them in a single file)
//! - C. Create separate messages (and store them in separate files)
//!
//! If you wish to use option A, use [`MultiGridGrib2Message`] or
//! [`MultiProductGrib2Message`] (or the low-level API) to include the set of
//! multiple elements in a single message. If you want to implement options B or
//! C, simply use [`SingleGrib2Message`] for each element. You can implement
//! option B by writing to the same output destination consecutively, and option
//! C by writing to separate output destinations.

use std::cell::{Ref, RefCell};

pub use complex::*;
pub use grid::*;
pub use message::*;
pub use simple::*;

use crate::def::grib2::template::param_set;

/// GPV data encoder.
pub struct GpvEncoder<'d> {
    data: std::borrow::Cow<'d, [f64]>,
    method: EncodingMethod,
    encoded: RefCell<Option<GpvEncodeOutput>>,
}

impl<'d> GpvEncoder<'d> {
    pub fn new(data: std::borrow::Cow<'d, [f64]>, method: EncodingMethod) -> Self {
        Self {
            data,
            method,
            encoded: RefCell::new(None),
        }
    }

    pub fn get_encoded(&'_ self) -> Ref<'_, GpvEncodeOutput> {
        if self.encoded.borrow().is_none() {
            *self.encoded.borrow_mut() = Some(self.encode());
        }

        Ref::map(self.encoded.borrow(), |c| c.as_ref().unwrap())
    }

    fn encode(&self) -> GpvEncodeOutput {
        let output = match &self.method {
            EncodingMethod::SimplePacking(simple_packing_strategy) => {
                let encoder = simple::Encoder::new(&self.data, simple_packing_strategy.clone());
                GpvEncodeOutputInner::SimplePacking(encoder.encode())
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
                GpvEncodeOutputInner::ComplexPacking(encoder.encode())
            }
        };
        GpvEncodeOutput(output)
    }
}

impl<'d> WriteGrib2PointValues for GpvEncoder<'d> {
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

/// Data obtained through GPV encoding. Instances are typically used to write
/// GRIB2 data via the methods defined in [`WriteGrib2DataSections`].
#[derive(Debug)]
pub struct GpvEncodeOutput(GpvEncodeOutputInner);

impl GpvEncodeOutput {
    /// Returns the parameter set.
    pub fn params(&self) -> GpvEncodeParams<'_> {
        match &self.0 {
            GpvEncodeOutputInner::SimplePacking(encoded) => {
                GpvEncodeParams::SimplePacking(encoded.params())
            }
            GpvEncodeOutputInner::ComplexPacking(encoded) => {
                let (simple, complex) = encoded.params();
                GpvEncodeParams::ComplexPacking(simple, complex)
            }
        }
    }
}

impl WriteGrib2DataSections for GpvEncodeOutput {
    fn section5_len(&self) -> usize {
        match &self.0 {
            GpvEncodeOutputInner::SimplePacking(encoded) => encoded.section5_len(),
            GpvEncodeOutputInner::ComplexPacking(encoded) => encoded.section5_len(),
        }
    }

    fn write_section5(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
        match &self.0 {
            GpvEncodeOutputInner::SimplePacking(encoded) => encoded.write_section5(buf),
            GpvEncodeOutputInner::ComplexPacking(encoded) => encoded.write_section5(buf),
        }
    }

    fn section6_len(&self) -> usize {
        match &self.0 {
            GpvEncodeOutputInner::SimplePacking(encoded) => encoded.section6_len(),
            GpvEncodeOutputInner::ComplexPacking(encoded) => encoded.section6_len(),
        }
    }

    fn write_section6(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
        match &self.0 {
            GpvEncodeOutputInner::SimplePacking(encoded) => encoded.write_section6(buf),
            GpvEncodeOutputInner::ComplexPacking(encoded) => encoded.write_section6(buf),
        }
    }

    fn section7_len(&self) -> usize {
        match &self.0 {
            GpvEncodeOutputInner::SimplePacking(encoded) => encoded.section7_len(),
            GpvEncodeOutputInner::ComplexPacking(encoded) => encoded.section7_len(),
        }
    }

    fn write_section7(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
        match &self.0 {
            GpvEncodeOutputInner::SimplePacking(encoded) => encoded.write_section7(buf),
            GpvEncodeOutputInner::ComplexPacking(encoded) => encoded.write_section7(buf),
        }
    }
}

#[non_exhaustive]
pub enum GpvEncodeParams<'a> {
    SimplePacking(&'a param_set::SimplePacking),
    ComplexPacking(&'a param_set::SimplePacking, &'a param_set::ComplexPacking),
}

#[derive(Debug)]
enum GpvEncodeOutputInner {
    SimplePacking(simple::Encoded),
    ComplexPacking(complex::Encoded),
}

// Since the name `Encode` is already in use on other branches currently under
// development, and since this trait is private, we'll continue to use that name
// for the time being.
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
