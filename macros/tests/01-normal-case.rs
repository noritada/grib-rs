use grib_proc_macros::parameter_codes;

#[parameter_codes(path = "tests/data/table")]
pub enum FooCodes {}

#[allow(dead_code)]
fn determine(value: FooCodes) -> ! {
    match value {
        FooCodes::TMP => todo!(),
        FooCodes::VTMP => todo!(),
        FooCodes::SOIL_M => todo!(),
        FooCodes::HGT => todo!(),
        FooCodes::U_GWD => todo!(),
        FooCodes::FiveWAVA => todo!(),
        FooCodes::FourLFTX => todo!(),
    }
}

fn main() {
    assert_eq!(FooCodes::HGT as u32, 0x_00_03_05);
}
