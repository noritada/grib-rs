use gen;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    let input_path = Path::new("def").join("CCT").join("xml").join("C11.xml");
    let output_path = Path::new(&out_dir).join("cct11.rs");
    let parsed = gen::cct11::parse(input_path);
    let built = gen::cct11::rebuild(parsed);
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
    let parsed = gen::cct00::parse(input_path);
    let built = gen::cct00::rebuild(parsed);
    fs::write(
        &output_path,
        format!(
            "pub const COMMON_CODE_TABLE_00: &'static [&'static str] = &{:#?};",
            built
        ),
    )
    .unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=def/CCT/C00.xml");
    println!("cargo:rerun-if-changed=def/CCT/C11.xml");
}
