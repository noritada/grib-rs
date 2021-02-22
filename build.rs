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
    let code_db = grib_build::grib2_codeflag::CodeDB::load(input_path);
    fs::write(
        &output_path,
        format!(
            "{}
            {}
            {}
            {}
            {}
            {}
            {}",
            code_db.export("0.0", "CODE_TABLE_0_0"),
            code_db.export("1.2", "CODE_TABLE_1_2"),
            code_db.export("1.3", "CODE_TABLE_1_3"),
            code_db.export("1.4", "CODE_TABLE_1_4"),
            code_db.export("3.1", "CODE_TABLE_3_1"),
            code_db.export("4.0", "CODE_TABLE_4_0"),
            code_db.export("5.0", "CODE_TABLE_5_0"),
        ),
    )
    .unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=def/CCT/xml/C00.xml");
    println!("cargo:rerun-if-changed=def/CCT/xml/C11.xml");
    println!("cargo:rerun-if-changed=def/GRIB2/xml/CodeFlag.xml");
}
