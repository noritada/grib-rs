use grib_proc_macros::parameter_codes;

#[parameter_codes]
pub enum FooCodes {}

#[allow(dead_code)]
fn determine(value: FooCodes) {
    match value {
        FooCodes::TMP => todo!(),
        FooCodes::VTMP => todo!(),
    }
}

fn main() {}
