use grib_codegen::parameter_codes;

#[parameter_codes(path = "tests/data/table")]
#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum FooCodes {}

#[allow(dead_code)]
fn determine(value: FooCodes) -> ! {
    match value {
        FooCodes::_TMP => todo!(),
        FooCodes::_VTMP => todo!(),
        FooCodes::_SOIL_M => todo!(),
        FooCodes::_HGT => todo!(),
        FooCodes::_U_GWD => todo!(),
        FooCodes::_5WAVA => todo!(),
        FooCodes::_260120 => todo!(),
        FooCodes::_4LFTX => todo!(),
        FooCodes::_CH3O2NO2 => todo!(),
    }
}

fn main() {
    assert_eq!(FooCodes::_HGT as u32, 0x_00_03_05);
    assert_eq!(FooCodes::try_from(0x_00_00_00), Ok(FooCodes::_TMP));
    assert_eq!(FooCodes::try_from(0x_00_03_10), Ok(FooCodes::_U_GWD));
    assert_eq!(FooCodes::try_from(0x_00_03_c2), Ok(FooCodes::_U_GWD));
    assert_eq!(FooCodes::try_from(0xffffffff), Err("code not found"));
    assert_eq!(format!("{:?}", FooCodes::_TMP), "_TMP");
    assert_eq!(FooCodes::_TMP, FooCodes::_TMP);
}
