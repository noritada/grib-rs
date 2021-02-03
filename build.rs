use grib_build;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    let input_path = Path::new("def").join("CCT").join("xml").join("C11.xml");
    let output_path = Path::new(&out_dir).join("cct11.rs");
    let parsed = grib_build::cct11::parse(input_path);
    let built = grib_build::cct11::rebuild(parsed);
    fs::write(
        &output_path,
        format!(
            "pub const COMMON_CODE_TABLE_11: &'static [&'static str] = &{:#?};",
            built
        ),
    )
    .unwrap();

    let input_path = Path::new("def").join("CCT").join("xml").join("C00.xml");
    let output_path = Path::new(&out_dir).join("cct00.rs");
    let parsed = grib_build::cct00::parse(input_path);
    let built = grib_build::cct00::rebuild(parsed);
    fs::write(
        &output_path,
        format!(
            "pub const COMMON_CODE_TABLE_00: &'static [&'static str] = &{:#?};",
            built
        ),
    )
    .unwrap();

    let input_path = Path::new("def")
        .join("GRIB2")
        .join("xml")
        .join("CodeFlag.xml");
    let output_path = Path::new(&out_dir).join("grib2_codeflag.rs");
    let code_db = grib_build::grib2_codeflag::CodeDB::new(input_path);
    fs::write(
        &output_path,
        format!(
            "pub const CODE_TABLE_1_2: &'static [&'static str] = &{:#?};
            pub const CODE_TABLE_1_3: &'static [&'static str] = &{:#?};
            pub const CODE_TABLE_1_4: &'static [&'static str] = &{:#?};",
            code_db.export("1.2"),
            code_db.export("1.3"),
            code_db.export("1.4")
        ),
    )
    .unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=def/CCT/C00.xml");
    println!("cargo:rerun-if-changed=def/CCT/C11.xml");
    println!("cargo:rerun-if-changed=def/GRIB2/xml/CodeFlag.xml");
}
