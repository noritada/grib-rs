use std::{env, fs, io::Read, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    if let Err(e) =
        check_nonemptiness(Path::new("def/CCT")).and(check_nonemptiness(Path::new("def/GRIB2")))
    {
        return Err(format!("{e}; run `git submodule update --init`").into());
    }

    let input_file_names = ["def/CCT/C00.csv", "def/CCT/C11.csv"];
    let mut db = grib_build::cct_csv::CodeDB::new();
    let output_path = Path::new(&out_dir).join("cct.rs");
    for path in &input_file_names {
        db.load(path)?;
        println!("cargo:rerun-if-changed={path}");
    }
    fs::write(output_path, format!("{db}"))?;

    let input_file_names = [
        "def/GRIB2/GRIB2_CodeFlag_0_0_CodeTable_en.csv",
        "def/GRIB2/GRIB2_CodeFlag_1_1_CodeTable_en.csv",
        "def/GRIB2/GRIB2_CodeFlag_1_2_CodeTable_en.csv",
        "def/GRIB2/GRIB2_CodeFlag_1_3_CodeTable_en.csv",
        "def/GRIB2/GRIB2_CodeFlag_1_4_CodeTable_en.csv",
        "def/GRIB2/GRIB2_CodeFlag_3_1_CodeTable_en.csv",
        "def/GRIB2/GRIB2_CodeFlag_4_0_CodeTable_en.csv",
        "def/GRIB2/GRIB2_CodeFlag_4_1_CodeTable_en.csv",
        "def/GRIB2/GRIB2_CodeFlag_4_2_CodeTable_en.csv",
        "def/GRIB2/GRIB2_CodeFlag_4_3_CodeTable_en.csv",
        "def/GRIB2/GRIB2_CodeFlag_4_4_CodeTable_en.csv",
        "def/GRIB2/GRIB2_CodeFlag_4_5_CodeTable_en.csv",
        "def/GRIB2/GRIB2_CodeFlag_5_0_CodeTable_en.csv",
    ];
    let mut db = grib_build::grib2_codeflag_csv::CodeDB::new();
    let output_path = Path::new(&out_dir).join("grib2_codeflag.rs");
    for path in &input_file_names {
        db.load(path)?;
        println!("cargo:rerun-if-changed={path}");
    }
    fs::write(output_path, format!("{db}"))?;

    let manifest = read_feature_docs()?;
    let output_path = Path::new(&out_dir).join("features.txt");
    fs::write(output_path, format!("{manifest}"))?;

    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}

fn check_nonemptiness(dir: &Path) -> Result<(), String> {
    dir.read_dir()
        .map_err(|_| format!("{} is not a directory", dir.to_string_lossy()))
        .and_then(|mut iter| {
            iter.next()
                .ok_or_else(|| format!("{} is empty", dir.to_string_lossy()))
                .map(|_| ())
        })
}

fn read_feature_docs() -> Result<String, Box<dyn std::error::Error>> {
    let mut f = std::fs::File::open("Cargo.toml")?;
    let mut manifest = String::new();
    f.read_to_string(&mut manifest)?;

    let doc = manifest.parse::<toml_edit::DocumentMut>()?;
    let features = doc["features"].to_string();
    let new_doc = format!(
        "```text
[features]
{features}
```"
    );
    println!("cargo:rerun-if-changed=Cargo.toml");

    Ok(new_doc)
}
