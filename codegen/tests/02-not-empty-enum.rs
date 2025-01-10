use grib_codegen::parameter_codes;

#[parameter_codes(path = "codegen/tests/data/table")]
pub enum FooCodes {
    Foo,
    Bar,
}

#[parameter_codes(path = "codegen/tests/data/table")]
pub struct BarCodes;

fn main() {}
