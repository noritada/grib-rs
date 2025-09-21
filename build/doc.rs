use std::{
    collections::HashMap,
    io::{self, Read},
    str::FromStr,
};

pub(crate) fn generate() -> Result<String, String> {
    let readme = read_readme()
        .map_err(|e| e.to_string())?
        .parse::<ReadMeSections>()?;
    let about = readme.get("About")?;
    let template_support = readme.get("Template support")?;
    let gds_template_support = readme.get("Supported grid definition templates")?;
    let drs_template_support = readme.get("Supported data representation templates")?;
    let example = readme.get("Usage example")?;

    let manifest = read_manifest()
        .map_err(|e| e.to_string())?
        .parse::<Features>()?
        .text();

    Ok(format!(
        "{about}

# Template support

{template_support}

## Supported grid definition templates

{gds_template_support}

## Supported data representation templates

{drs_template_support}

# Example

{example}

# Crate features

Following crate features are available in this crate. These descriptions are
extracted from the manifest of the crate.

{manifest}
"
    ))
}

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

fn read_readme() -> io::Result<String> {
    let mut f = std::fs::File::open("README.md")?;
    let mut readme = String::new();
    f.read_to_string(&mut readme)?;

    println!("cargo:rerun-if-changed=README.md");
    Ok(readme)
}

#[derive(Debug)]
struct ReadMeSections(HashMap<String, String>);

impl ReadMeSections {
    fn get(&self, key: &str) -> Result<&String, String> {
        self.0
            .get(key)
            .ok_or(format!(r#""{key}" section not found"#))
    }
}

impl std::str::FromStr for ReadMeSections {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Line terminators are not included in the lines returned by the iterator from
        // `str::lines`.
        let mut lines = s.split_inclusive("\n");

        let mut map = HashMap::<String, String>::new();
        let mut start = 0;
        let mut pos = 0;
        let mut title: Option<&str> = None;

        let push = |map: &mut HashMap<String, String>,
                    title: Option<&str>,
                    s: &str,
                    start: usize,
                    end: usize| {
            let Some(title) = title else { return };
            if end <= start {
                return;
            }
            let content = s[start..end].to_owned();
            map.insert(title.to_owned(), content);
        };

        loop {
            let Some(line) = lines.next() else {
                push(&mut map, title, s, start, pos);
                break;
            };
            pos += line.len();

            if line.starts_with("#") {
                push(&mut map, title, s, start, pos - line.len());
                let s = line.trim_start_matches('#').trim();
                title = Some(s);
                start = pos;
            }
        }

        Ok(Self(map))
    }
}
