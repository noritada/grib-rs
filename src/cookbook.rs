//! A cookbook of examples for GRIB2 data handling.
//!
//! # Table of contents
//!
//! 1. [Comparison of various GRIB2 data library/tool operations][cmp]
//!     * [Listing all submessages inside][cmp-listing]
//!     * [Finding submessages inside that match some condition][cmp-finding]
//!     * [Extracting values with location info from a submessage][cmp-decoding]
//!
//! [cmp]: #comparison-of-various-grib2-data-librarytool-operations
//! [cmp-listing]: #listing-all-submessages-inside
//! [cmp-finding]: #finding-submessages-inside-that-match-some-condition
//! [cmp-decoding]: #extracting-values-with-location-info-from-a-submessage
//!
//! # Comparison of various GRIB2 data library/tool operations
//!
//! This section provides example code of data operations using some GRIB
//! processing libraries and tools.
//!
//! ## Listing all submessages inside
//!
//! GRIB tools from ecCodes:
//!
//! ```shell
//! $ grib_ls datafile.grib
//! ```
//!
//! wgrib2:
//!
//! ```shell
//! $ wgrib2 datafile.grib
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
//! use std::{fs::File, io::BufReader, path::Path};
//!
//! use grib::codetables::{CodeTable4_2, Lookup};
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
//!     for (_index, submessage) in grib2.iter() {
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
//! $ gribber list datafile.grib
//! ```
//!
//! ## Finding submessages inside that match some condition
//!
//! GRIB tools from ecCodes:
//!
//! ```shell
//! $ grib_ls -w forecastTime=0,level=850,shortName=u datafile.grib
//! ```
//!
//! wgrib2:
//!
//! ```shell
//! $ wgrib2 datafile.grib -match ':3 hour fcst:'
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
//! use std::{fs::File, io::BufReader, path::Path};
//!
//! use grib::{codetables::grib2::*, ForecastTime, Name};
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
//!     for (index, submessage) in grib2.iter() {
//!         let ft = submessage.prod_def().forecast_time();
//!         match ft {
//!             Some(ForecastTime {
//!                 unit: Name(Table4_4::Hour),
//!                 value: hours,
//!             }) => {
//!                 if hours == forecast_time_hours {
//!                     println!("{}.{}: {}", index.0, index.1, hours);
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
//! $ gribber list datafile.grib | grep '3 Hour'
//! ```
//!
//! (gribber's API for finding submessages is still in the conceptual stage and
//! is not yet available.)
//!
//! ## Extracting values with location info from a submessage
//!
//! GRIB tools from ecCodes:
//!
//! ```shell
//! $ grib_get_data -w forecastTime=0,count=1 datafile.grib
//! ```
//!
//! wgrib2 (creating a flat binary file of values):
//!
//! ```shell
//! $ wgrib2 -d 1.1 -order we:ns -no_header -bin output.bin datafile.grib
//! ```
//!
//! pygrib:
//!
//! ```python
//! import pygrib
//!
//! path = "datafile.grib"
//! grib = pygrib.open(path)
//! submessage = grib.message(1)
//! lats, lons = submessage.latlons()
//! values = submessage.values
//! print((lats, lons, values))
//! ```
//!
//! grib-rs:
//!
//! ```rust
//! use std::{
//!     fs::File,
//!     io::{BufReader, Read, Write},
//!     path::Path,
//! };
//!
//! use grib::codetables::{grib2::*, *};
//!
//! fn decode_layer<P>(path: P, message_index: (usize, usize))
//! where
//!     P: AsRef<Path>,
//! {
//!     let f = File::open(path).unwrap();
//!     let f = BufReader::new(f);
//!
//!     let grib2 = grib::from_reader(f).unwrap();
//!     let (_index, submessage) = grib2
//!         .iter()
//!         .find(|(index, _)| *index == message_index)
//!         .ok_or("no such index")
//!         .unwrap();
//!
//!     let latlons = submessage.latlons().unwrap();
//!     let decoder = grib::Grib2SubmessageDecoder::from(submessage).unwrap();
//!     let values = decoder.dispatch().unwrap();
//!
//!     for ((lat, lon), value) in latlons.zip(values) {
//!         println!("{lat} {lon} {value}");
//!     }
//! }
//!
//! fn main() {
//!     let path = "testdata/gdas.t12z.pgrb2.0p25.f000.0-10.xz";
//!
//!     let mut buf = Vec::new();
//!     let mut out = tempfile::NamedTempFile::new().unwrap();
//!
//!     let f = File::open(path).unwrap();
//!     let f = BufReader::new(f);
//!     let mut f = xz2::bufread::XzDecoder::new(f);
//!     f.read_to_end(&mut buf).unwrap();
//!     out.write_all(&buf).unwrap();
//!
//!     decode_layer(&out.path(), (0, 0));
//! }
//! ```
//!
//! gribber (showing values along with lat/lon info):
//!
//! ```shell
//! $ gribber decode datafile.grib 0.0
//! ```
//!
//! gribber (creating a flat binary file of values):
//!
//! ```shell
//! $ gribber decode -b output.bin datafile.grib 0.0
//! ```
