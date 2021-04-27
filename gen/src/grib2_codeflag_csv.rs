use serde::Deserialize;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::path::PathBuf;

use crate::*;

#[derive(Debug, Deserialize)]
struct Record {
    #[serde(rename = "Title_en")]
    title: String,
    #[serde(rename = "SubTitle_en")]
    subtitle: String,
    #[serde(rename = "CodeFlag")]
    code_flag: String,
    #[serde(rename = "Value")]
    value: String,
    #[serde(rename = "MeaningParameterDescription_en")]
    description: String,
    #[serde(rename = "Note_en")]
    note: String,
    #[serde(rename = "UnitComments_en")]
    unit: String,
    #[serde(rename = "Status")]
    status: String,
}

pub struct CodeDB {
    data: BTreeMap<(u8, u8), CodeTable>,
}

impl CodeDB {
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    pub fn load(&mut self, path: PathBuf) -> Result<(), Box<dyn Error>> {
        let basename = path.file_stem().ok_or(PathError)?.to_string_lossy();
        let words: Vec<_> = basename.split("_").take(4).collect();
        if let ["GRIB2", "CodeFlag", section, number] = words[..] {
            self.data.insert(
                (section.parse::<u8>()?, number.parse::<u8>()?),
                Self::parse_file(path)?,
            );
        };

        Ok(())
    }

    pub fn parse_file(path: PathBuf) -> Result<CodeTable, Box<dyn Error>> {
        let f = File::open(&path)?;
        let mut reader = csv::Reader::from_reader(f);
        let mut iter = reader.deserialize();

        let record = iter.next().ok_or("CSV does not have a body")?;
        let record: Record = record?;
        let mut codetable =
            CodeTable::new_with((record.code_flag, record.description), record.title);

        for record in iter {
            let record: Record = record?;
            codetable.data.push((record.code_flag, record.description));
        }

        Ok(codetable)
    }

    pub fn export(&self, id: (u8, u8)) -> String {
        match self.get(id) {
            Some(code_table) => {
                let variable_name = self.get_variable_name(id);
                code_table.export(&variable_name)
            }
            None => "[]".to_string(),
        }
    }

    fn get_variable_name(&self, id: (u8, u8)) -> String {
        format!("CODE_TABLE_{}_{}", id.0, id.1)
    }

    pub fn get(&self, id: (u8, u8)) -> Option<&CodeTable> {
        self.data.get(&id)
    }
}

impl fmt::Display for CodeDB {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for (id, code_table) in &self.data {
            if first {
                first = false;
            } else {
                write!(f, "\n\n")?;
            }

            let variable_name = self.get_variable_name(*id);
            write!(f, "{}", code_table.export(&variable_name))?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct PathError;

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PathError")
    }
}

impl Error for PathError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::Path;

    #[test]
    fn parse_file() {
        let path = Path::new("testdata").join("GRIB2_CodeFlag_0_0_CodeTable_no_subtitle.csv");
        let table = CodeDB::parse_file(path).unwrap();
        assert_eq!(table.desc, "Foo");
        assert_eq!(
            table.data,
            vec![
                ("0", "0A"),
                ("1", "0B"),
                ("2-254", "Reserved"),
                ("255", "Missing"),
            ]
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect::<Vec<_>>()
        );
    }

    #[test]
    fn codedb_getting_code_table() {
        let mut db = CodeDB::new();
        let pair = ("0".to_owned(), "0".to_owned());
        let table = CodeTable::new_with(pair, "".to_owned());
        db.data.insert((0, 0), table.clone());
        assert_eq!(db.get((0, 0)).unwrap(), &table);
    }

    #[test]
    fn export() {
        let path = Path::new("testdata").join("GRIB2_CodeFlag_0_0_CodeTable_no_subtitle.csv");
        let mut db = CodeDB::new();
        db.load(path).unwrap();
        assert_eq!(
            db.export((0, 0)),
            "\
/// Foo
pub const CODE_TABLE_0_0: &'static [&'static str] = &[
    \"0A\",
    \"0B\",
];"
        );
    }

    #[test]
    fn format() {
        let mut db = CodeDB::new();
        db.load(PathBuf::from(
            "testdata/GRIB2_CodeFlag_1_0_CodeTable_no_subtitle.csv",
        ))
        .unwrap();
        db.load(PathBuf::from(
            "testdata/GRIB2_CodeFlag_0_0_CodeTable_no_subtitle.csv",
        ))
        .unwrap();
        assert_eq!(
            format!("{}", db),
            "\
/// Foo
pub const CODE_TABLE_0_0: &'static [&'static str] = &[
    \"0A\",
    \"0B\",
];

/// Bar
pub const CODE_TABLE_1_0: &'static [&'static str] = &[
    \"1A\",
    \"1B\",
];"
        );
    }

    #[test]
    fn codetable_to_vec() {
        let path = Path::new("testdata").join("GRIB2_CodeFlag_0_0_CodeTable_no_subtitle.csv");
        let mut db = CodeDB::new();
        db.load(path).unwrap();
        assert_eq!(db.get((0, 0)).unwrap().to_vec(), vec!["0A", "0B",]);
    }
}
