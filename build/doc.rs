use std::{
    io::{self, Read},
    str::FromStr,
};

pub(crate) fn read_manifest() -> io::Result<String> {
    let mut f = std::fs::File::open("Cargo.toml")?;
    let mut manifest = String::new();
    f.read_to_string(&mut manifest)?;

    println!("cargo:rerun-if-changed=Cargo.toml");
    Ok(manifest)
}

pub(crate) struct Features(String);

impl Features {
    pub(crate) fn text(self) -> String {
        self.0
    }
}

impl FromStr for Features {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let doc = s
            .parse::<toml_edit::DocumentMut>()
            .map_err(|e| e.to_string())?;
        let features = doc["features"].to_string();
        let new_doc = format!(
            "```text
[features]
{features}
```"
        );

        Ok(Self(new_doc))
    }
}
