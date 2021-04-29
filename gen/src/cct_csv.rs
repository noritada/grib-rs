use serde::Deserialize;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::path::PathBuf;

use crate::*;

#[derive(Debug, Deserialize)]
struct C00Record {
    #[serde(rename = "GRIB version number")]
    grib_version: String,
    #[serde(rename = "BUFR version number")]
    bufr_version: String,
    #[serde(rename = "CREX version number")]
    crex_version: String,
    #[serde(rename = "Effective date")]
    date: String,
    #[serde(rename = "Status")]
    status: String,
}

pub struct CodeDB {
    data: BTreeMap<u8, CodeTable>,
}

impl CodeDB {
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    pub fn load(&mut self, path: PathBuf) -> Result<(), Box<dyn Error>> {
        let basename = path.file_stem().ok_or("unexpected path")?.to_string_lossy();
        match &*basename {
            "C00" => {
                self.data.insert(0, Self::parse_file_c00(path)?);
            }
            _ => {}
        }

        Ok(())
    }

    pub fn parse_file_c00(path: PathBuf) -> Result<CodeTable, Box<dyn Error>> {
        let f = File::open(&path)?;
        let mut reader = csv::Reader::from_reader(f);
        let mut iter = reader.deserialize();

        let record = iter.next().ok_or("CSV does not have a body")?;
        let record: C00Record = record?;
        let mut codetable = CodeTable::new_with(
            (record.grib_version, record.date),
            "Common Code Table C-0".to_owned(),
        );

        for record in iter {
            let record: C00Record = record?;
            codetable.data.push((record.grib_version, record.date));
        }

        Ok(codetable)
    }

    pub fn export(&self, id: u8) -> String {
        match self.get(id) {
            Some(code_table) => {
                let variable_name = self.get_variable_name(id);
                code_table.export(&variable_name)
            }
            None => "[]".to_string(),
        }
    }

    fn get_variable_name(&self, id: u8) -> String {
        format!("COMMON_CODE_TABLE_{:02}", id)
    }

    pub fn get(&self, id: u8) -> Option<&CodeTable> {
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

#[cfg(test)]
mod tests {
    use super::*;

    const PATH_STR_C00: &str = "testdata/C00.csv";

    #[test]
    fn parse_file_c00() {
        let path = PathBuf::from(PATH_STR_C00);
        let table = CodeDB::parse_file_c00(path).unwrap();
        assert_eq!(table.desc, "Common Code Table C-0");
        assert_eq!(
            table.data,
            vec![
                ("0", "Experimental"),
                ("", "1 January 1998"),
                ("", "1 January 1999"),
                ("1", "1 January 2000"),
                ("2", "1 January 2001"),
                ("3", "Pre-operational to be implemented by next amendment"),
                ("4-254", "Future versions"),
                ("255", "Missing"),
            ]
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect::<Vec<_>>()
        );
    }

    #[test]
    fn export_c00() {
        let mut db = CodeDB::new();
        db.load(PathBuf::from(PATH_STR_C00)).unwrap();
        assert_eq!(
            db.export(0),
            "\
/// Common Code Table C-0
pub const COMMON_CODE_TABLE_00: &'static [&'static str] = &[
    \"Experimental\",
    \"1 January 2000\",
    \"1 January 2001\",
    \"Pre-operational to be implemented by next amendment\",
];"
        );
    }

    #[test]
    fn format() {
        let mut db = CodeDB::new();
        db.load(PathBuf::from(PATH_STR_C00)).unwrap();
        assert_eq!(
            format!("{}", db),
            "\
/// Common Code Table C-0
pub const COMMON_CODE_TABLE_00: &'static [&'static str] = &[
    \"Experimental\",
    \"1 January 2000\",
    \"1 January 2001\",
    \"Pre-operational to be implemented by next amendment\",
];"
        );
    }

    #[test]
    fn codetable_to_vec() {
        let mut db = CodeDB::new();
        db.load(PathBuf::from(PATH_STR_C00)).unwrap();
        assert_eq!(
            db.get(0).unwrap().to_vec(),
            vec![
                "Experimental",
                "1 January 2000",
                "1 January 2001",
                "Pre-operational to be implemented by next amendment",
            ]
        );
    }
}
