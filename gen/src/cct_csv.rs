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

#[derive(Debug, Deserialize)]
struct C11Record {
    #[serde(rename = "CREX2")]
    crex2: String,
    #[serde(rename = "GRIB2_BUFR4")]
    grib2_bufr4: String,
    #[serde(rename = "OriginatingGeneratingCentre_en")]
    center: String,
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
            "C11" => {
                self.data.insert(11, Self::parse_file_c11(path)?);
            }
            _ => {}
        }

        Ok(())
    }

    pub fn parse_file_c00(path: PathBuf) -> Result<CodeTable, Box<dyn Error>> {
        let f = File::open(&path)?;
        let mut reader = csv::Reader::from_reader(f);
        let mut codetable = CodeTable::new("Common Code Table C-0".to_owned());

        for record in reader.deserialize() {
            let record: C00Record = record?;
            codetable.data.push((record.grib_version, record.date));
        }

        Ok(codetable)
    }

    pub fn parse_file_c11(path: PathBuf) -> Result<CodeTable, Box<dyn Error>> {
        let f = File::open(&path)?;
        let mut reader = csv::Reader::from_reader(f);
        let mut codetable = CodeTable::new("Common Code Table C-11".to_owned());

        for record in reader.deserialize() {
            let record: C11Record = record?;
            codetable.data.push((record.grib2_bufr4, record.center));
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
    const PATH_STR_C11: &str = "testdata/C11.csv";

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
    fn parse_file_c11() {
        let path = PathBuf::from(PATH_STR_C11);
        let table = CodeDB::parse_file_c11(path).unwrap();
        assert_eq!(table.desc, "Common Code Table C-11");
        assert_eq!(
            table.data,
            vec![
                ("0", "A"),
                ("", "Comment"),
                ("1", "B"),
                ("2", ")"),
                ("3", "C"),
                ("4", "Reserved"),
                ("5", "D"),
                ("6", "Reserved for other centres"),
                ("7-9", "E"),
                ("10-14", "Reserved"),
                ("15", "F"),
                ("16-65534", "Reserved for other centres"),
                ("65535", "Missing value"),
                ("Not applicable", "Not used"),
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
const COMMON_CODE_TABLE_00: &'static [&'static str] = &[
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
        db.load(PathBuf::from(PATH_STR_C11)).unwrap();
        assert_eq!(
            format!("{}", db),
            "\
/// Common Code Table C-0
const COMMON_CODE_TABLE_00: &'static [&'static str] = &[
    \"Experimental\",
    \"1 January 2000\",
    \"1 January 2001\",
    \"Pre-operational to be implemented by next amendment\",
];

/// Common Code Table C-11
const COMMON_CODE_TABLE_11: &'static [&'static str] = &[
    \"A\",
    \"B\",
    \"\",
    \"C\",
    \"\",
    \"D\",
    \"\",
    \"E\",
    \"E\",
    \"E\",
    \"\",
    \"\",
    \"\",
    \"\",
    \"\",
    \"F\",
];"
        );
    }

    #[test]
    fn codetable_to_vec() {
        let mut db = CodeDB::new();
        db.load(PathBuf::from(PATH_STR_C00)).unwrap();
        db.load(PathBuf::from(PATH_STR_C11)).unwrap();
        assert_eq!(
            db.get(0).unwrap().to_vec(),
            vec![
                "Experimental",
                "1 January 2000",
                "1 January 2001",
                "Pre-operational to be implemented by next amendment",
            ]
        );
        assert_eq!(
            db.get(11).unwrap().to_vec(),
            vec!["A", "B", "", "C", "", "D", "", "E", "E", "E", "", "", "", "", "", "F",]
        );
    }
}
