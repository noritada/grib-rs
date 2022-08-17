//!
//! # Comparison of GRIB2 data operation libraries/tools
//!
//! This section provides example code of data operations using some GRIB
//! processing libraries and tools.
//!
//! ## Listing all submessages inside a GRIB message
//!
//! wgrib2:
//!
//! ```shell
//! wgrib2 datafile.grib
//! ```
//!
//! pygrib:
//!
//! ```python
//! import pygrib
//!
//! path = "testdata/icon_global_icosahedral_single-level_2021112018_000_TOT_PREC.grib2"
//! grib = pygrib.open(path)
//! for submessage in grib:
//!     print(submessage)
//! ```
//!
//! grib-rs:
//!
//! ```rust
//! use grib::codetables::{CodeTable4_2, Lookup};
//! use std::fs::File;
//! use std::io::BufReader;
//! use std::path::Path;
//!
//! fn list_submessages<P>(path: P)
//! where
//!     P: AsRef<Path>,
//! {
//!     let f = File::open(path).unwrap();
//!     let f = BufReader::new(f);
//!
//!     let grib2 = grib::from_reader(f).unwrap();
//!
//!     for submessage in grib2.iter() {
//!         let discipline = submessage.indicator().discipline;
//!         let category = submessage.prod_def().parameter_category().unwrap();
//!         let parameter = submessage.prod_def().parameter_number().unwrap();
//!         let parameter = CodeTable4_2::new(discipline, category).lookup(usize::from(parameter));
//!
//!         let forecast_time = submessage.prod_def().forecast_time().unwrap();
//!
//!         let (first, _second) = submessage.prod_def().fixed_surfaces().unwrap();
//!         let elevation_level = first.value();
//!
//!         println!(
//!             "{:<31} {:>14} {:>17}",
//!             parameter.to_string(),
//!             forecast_time.to_string(),
//!             elevation_level
//!         );
//!     }
//! }
//!
//! fn main() {
//!     let path = "testdata/icon_global_icosahedral_single-level_2021112018_000_TOT_PREC.grib2";
//!     list_submessages(&path);
//! }
//! ```
//!
//! gribber:
//!
//! ```shell
//! gribber list datafile.grib
//! ```
//!
//! ## Finding submessages that match some conditions inside a GRIB message
//!
//! wgrib2:
//!
//! ```shell
//! wgrib2 datafile.grib -match ':3 hour fcst:'
//! ```
//!
//! pygrib:
//!
//! ```python
//! import pygrib
//!
//! path = "testdata/icon_global_icosahedral_single-level_2021112018_000_TOT_PREC.grib2"
//! grib = pygrib.open(path)
//! for submessage in grib.select(forecastTime=3):
//!     print(submessage)
//! ```
//!
//! grib-rs:
//!
//! ```rust
//! use grib::codetables::grib2::*;
//! use grib::codetables::*;
//! use grib::datatypes::*;
//! use std::fs::File;
//! use std::io::BufReader;
//! use std::path::Path;
//!
//! fn find_submessages<P>(path: P, forecast_time_hours: u32)
//! where
//!     P: AsRef<Path>,
//! {
//!     let f = File::open(path).unwrap();
//!     let f = BufReader::new(f);
//!
//!     let grib2 = grib::from_reader(f).unwrap();
//!
//!     for (index, submessage) in grib2.iter().enumerate() {
//!         let ft = submessage.prod_def().forecast_time();
//!         match ft {
//!             Some(ForecastTime {
//!                 unit: Name(Table4_4::Hour),
//!                 value: hours,
//!             }) => {
//!                 if hours == forecast_time_hours {
//!                     println!("{}: {}", index, hours);
//!                 }
//!             }
//!             _ => {}
//!         }
//!     }
//! }
//!
//! fn main() {
//!     let path = "testdata/icon_global_icosahedral_single-level_2021112018_000_TOT_PREC.grib2";
//!     find_submessages(&path, 3);
//! }
//! ```
//!
//! gribber:
//!
//! ```shell
//! gribber list datafile.grib | grep '3 Hour'
//! ```
//!
//! (gribber's API for finding submessages is still in the conceptual stage and
//! is not yet available.)
