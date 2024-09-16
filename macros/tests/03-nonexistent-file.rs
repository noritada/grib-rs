use grib_proc_macros::parameter_codes;

#[parameter_codes(path = "nosuchfile")]
pub enum FooCodes {}

fn main() {}
