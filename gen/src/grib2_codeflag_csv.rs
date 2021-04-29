use serde::Deserialize;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;

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

#[derive(Debug, PartialEq, Eq)]
struct Category(Option<(u8, u8)>);

impl FromStr for Category {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "" => Ok(Category(None)),
            s => {
                let mut splitted = s.split(", ");

                let first = splitted.next().ok_or(ParseError)?;
                let words: Vec<_> = first.split(" ").take(3).collect();
                let discipline = match words[..] {
                    ["Product", "discipline", num] => u8::from_str(num).map_err(|_| ParseError),
                    _ => Err(ParseError),
                }?;

                let second = splitted.next().ok_or(ParseError)?;
                let second = second.split(":").next().ok_or(ParseError)?;
                let words: Vec<_> = second.split(" ").take(3).collect();
                let parameter = match words[..] {
                    ["parameter", "category", num] => u8::from_str(num).map_err(|_| ParseError),
                    _ => Err(ParseError),
                }?;

                Ok(Category(Some((discipline, parameter))))
            }
        }
    }
}

impl CodeDB {
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    pub fn load(&mut self, path: PathBuf) -> Result<(), Box<dyn Error>> {
        let basename = path.file_stem().ok_or("unexpected path")?.to_string_lossy();
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
pub struct ParseError;

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ParseError")
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PATH_STR_0: &str = "testdata/GRIB2_CodeFlag_0_0_CodeTable_no_subtitle.csv";
    const PATH_STR_1: &str = "testdata/GRIB2_CodeFlag_1_0_CodeTable_no_subtitle.csv";

    #[test]
    fn parse_subtitle() {
        let string =
            "Product discipline 0 - Meteorological products, parameter category 0: temperature";
        let category = string.parse::<Category>().unwrap();
        assert_eq!(category, Category(Some((0, 0))));
    }

    #[test]
    fn parse_file() {
        let path = PathBuf::from(PATH_STR_0);
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
    fn export() {
        let mut db = CodeDB::new();
        db.load(PathBuf::from(PATH_STR_0)).unwrap();
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
        db.load(PathBuf::from(PATH_STR_0)).unwrap();
        db.load(PathBuf::from(PATH_STR_1)).unwrap();
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
        let mut db = CodeDB::new();
        db.load(PathBuf::from(PATH_STR_0)).unwrap();
        assert_eq!(db.get((0, 0)).unwrap().to_vec(), vec!["0A", "0B",]);
    }
}
