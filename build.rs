use grib_build;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    let input_file_names = ["def/CCT/C00.csv", "def/CCT/C11.csv"];
    let mut db = grib_build::cct_csv::CodeDB::new();
    let output_path = Path::new(&out_dir).join("cct.rs");
    for file_name in &input_file_names {
        let path = PathBuf::from(file_name);
        db.load(path).unwrap();
        println!("cargo:rerun-if-changed={}", file_name);
    }
    fs::write(&output_path, format!("{}", db)).unwrap();

    let input_file_names = [
        "def/GRIB2/GRIB2_CodeFlag_0_0_CodeTable_en.csv",
        "def/GRIB2/GRIB2_CodeFlag_1_1_CodeTable_en.csv",
        "def/GRIB2/GRIB2_CodeFlag_1_2_CodeTable_en.csv",
        "def/GRIB2/GRIB2_CodeFlag_1_3_CodeTable_en.csv",
        "def/GRIB2/GRIB2_CodeFlag_1_4_CodeTable_en.csv",
        "def/GRIB2/GRIB2_CodeFlag_3_1_CodeTable_en.csv",
        "def/GRIB2/GRIB2_CodeFlag_4_1_CodeTable_en.csv",
        "def/GRIB2/GRIB2_CodeFlag_4_2_CodeTable_en.csv",
        "def/GRIB2/GRIB2_CodeFlag_4_0_CodeTable_en.csv",
        "def/GRIB2/GRIB2_CodeFlag_5_0_CodeTable_en.csv",
    ];
    let mut db = grib_build::grib2_codeflag_csv::CodeDB::new();
    let output_path = Path::new(&out_dir).join("grib2_codeflag.rs");
    for file_name in &input_file_names {
        let path = PathBuf::from(file_name);
        db.load(path).unwrap();
        println!("cargo:rerun-if-changed={}", file_name);
    }
    fs::write(&output_path, format!("{}", db)).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
}
