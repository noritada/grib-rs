use grib_proc_macros::parameter_codes;

#[parameter_codes(path = "tests/data/table")]
pub enum FooCodes {
    Foo,
    Bar,
}

#[parameter_codes(path = "tests/data/table")]
pub struct BarCodes;

fn main() {}
