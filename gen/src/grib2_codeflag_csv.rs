use std::{collections::BTreeMap, error::Error, fmt, fs::File, path::Path, str::FromStr};

use serde::Deserialize;

use crate::CodeTable;

#[derive(Debug, Deserialize)]
struct Record {
    #[serde(rename = "Title_en")]
    title: String,
    #[serde(rename = "SubTitle_en")]
    subtitle: String,
    #[serde(rename = "CodeFlag")]
    code_flag: String,
    #[allow(dead_code)]
    #[serde(rename = "Value")]
    value: String,
    #[serde(rename = "MeaningParameterDescription_en")]
    description: String,
    #[allow(dead_code)]
    #[serde(rename = "Note_en")]
    note: String,
    #[allow(dead_code)]
    #[serde(rename = "UnitComments_en")]
    unit: String,
    #[allow(dead_code)]
    #[serde(rename = "Status")]
    status: String,
}

pub struct CodeDB {
    data: BTreeMap<(u8, u8, OptArg), CodeTable>,
}

impl Default for CodeDB {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeDB {
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    pub fn load<P>(&mut self, path: P) -> Result<(), Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let basename = path
            .as_ref()
            .file_stem()
            .ok_or("unexpected path")?
            .to_string_lossy();
        let words: Vec<_> = basename.split('_').take(4).collect();
        if let ["GRIB2", "CodeFlag", section, number] = words[..] {
            let section = section.parse::<u8>()?;
            let number = number.parse::<u8>()?;
            for (category, table) in Self::parse_file(path)? {
                self.data.insert((section, number, category), table);
            }
        };

        Ok(())
    }

    fn parse_file<P>(path: P) -> Result<Vec<(OptArg, CodeTable)>, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let f = File::open(path)?;
        let mut reader = csv::Reader::from_reader(f);
        let mut out_tables = Vec::<(OptArg, CodeTable)>::new();

        for record in reader.deserialize() {
            let record: Record = record?;
            let category = record.subtitle.parse::<OptArg>()?;

            let current = out_tables.last();
            if current.is_none() || category != current.unwrap().0 {
                let new_codetable = CodeTable::new(record.title);
                out_tables.push((category, new_codetable));
            }

            out_tables
                .last_mut()
                .unwrap()
                .1
                .data
                .push((record.code_flag, record.description));
        }

        Ok(out_tables)
    }

    pub fn export(&self, id: (u8, u8, OptArg)) -> String {
        match self.get(id) {
            Some(code_table) => {
                let variable_name = self.get_variable_name(id);
                code_table.export(&variable_name)
            }
            None => "[]".to_string(),
        }
    }

    fn get_variable_name(&self, id: (u8, u8, OptArg)) -> String {
        match id {
            (section, number, OptArg::None) => format!("CODE_TABLE_{section}_{number}"),
            (section, number, OptArg::L1(discipline)) => {
                format!("CODE_TABLE_{section}_{number}_{discipline}")
            }
            (section, number, OptArg::L2(discipline, category)) => {
                format!("CODE_TABLE_{section}_{number}_{discipline}_{category}")
            }
        }
    }

    pub fn get(&self, id: (u8, u8, OptArg)) -> Option<&CodeTable> {
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

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub enum OptArg {
    None,
    L1(u8),
    L2(u8, u8),
}

impl FromStr for OptArg {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "" => Ok(OptArg::None),
            s => {
                let mut splitted = s.split(", ");

                let first = splitted.next().ok_or(ParseError)?;
                let words: Vec<_> = first.split(' ').take(3).collect();
                let discipline = match words[..] {
                    ["Product", "discipline", num] => u8::from_str(num).map_err(|_| ParseError),
                    _ => Err(ParseError),
                }?;

                if let Some(second) = splitted.next() {
                    let second = second.split(':').next().ok_or(ParseError)?;
                    let words: Vec<_> = second.split(' ').take(3).collect();
                    let parameter = match words[..] {
                        ["parameter", "category", num] => u8::from_str(num).map_err(|_| ParseError),
                        _ => Err(ParseError),
                    }?;

                    Ok(OptArg::L2(discipline, parameter))
                } else {
                    Ok(OptArg::L1(discipline))
                }
            }
        }
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
    const PATH_STR_2: &str = "testdata/GRIB2_CodeFlag_4_1_CodeTable_with_subtitle.csv";
    const PATH_STR_3: &str = "testdata/GRIB2_CodeFlag_4_2_CodeTable_with_subtitle.csv";

    #[test]
    fn parse_subtitle() {
        let string =
            "Product discipline 0 - Meteorological products, parameter category 0: temperature";
        let category = string.parse::<OptArg>().unwrap();
        assert_eq!(category, OptArg::L2(0, 0));
    }

    #[test]
    fn parse_file_no_subtitle() {
        let tables = CodeDB::parse_file(PATH_STR_0).unwrap();

        let expected_title = "Foo".to_owned();
        let expected_data = vec![(
            OptArg::None,
            vec![
                ("0", "0A"),
                ("1", "0B"),
                ("2-254", "Reserved"),
                ("255", "Missing"),
            ],
        )];

        let expected = expected_data
            .iter()
            .map(|(c, table)| {
                (
                    *c,
                    CodeTable {
                        desc: expected_title.clone(),
                        data: table
                            .iter()
                            .map(|(a, b)| (a.to_string(), b.to_string()))
                            .collect::<Vec<_>>(),
                    },
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(tables, expected);
    }

    #[test]
    fn parse_file_with_subtitle_l1() {
        let tables = CodeDB::parse_file(PATH_STR_2).unwrap();

        let expected_title = "Baz".to_owned();
        let expected_data = vec![
            (
                OptArg::L1(0),
                vec![
                    ("0", "Temperature"),
                    ("1-2", "Reserved"),
                    ("3", "Mass"),
                    ("4-191", "Reserved"),
                    ("192-254", "Reserved for local use"),
                    ("255", "Missing"),
                ],
            ),
            (
                OptArg::L1(3),
                vec![
                    ("0", "Image format products"),
                    ("1", "Quantitative products"),
                    ("2", "Cloud properties"),
                    ("3-191", "Reserved"),
                    ("192-254", "Reserved for local use"),
                    ("255", "Missing"),
                ],
            ),
            (
                OptArg::L1(20),
                vec![
                    ("0", "Health indicators"),
                    ("1-191", "Reserved"),
                    ("192-254", "Reserved for local use"),
                    ("255", "Missing"),
                ],
            ),
        ];

        let expected = expected_data
            .iter()
            .map(|(c, table)| {
                (
                    *c,
                    CodeTable {
                        desc: expected_title.clone(),
                        data: table
                            .iter()
                            .map(|(a, b)| (a.to_string(), b.to_string()))
                            .collect::<Vec<_>>(),
                    },
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(tables, expected);
    }

    #[test]
    fn parse_file_with_subtitle_l2() {
        let tables = CodeDB::parse_file(PATH_STR_3).unwrap();

        let expected_title = "Baz".to_owned();
        let expected_data = vec![
            (
                OptArg::L2(0, 0),
                vec![
                    ("0", "Temperature"),
                    ("1-9", "Reserved"),
                    ("10", "Latent heat net flux"),
                    ("11-191", "Reserved"),
                    ("192-254", "Reserved for local use"),
                    ("255", "Missing"),
                ],
            ),
            (
                OptArg::L2(0, 191),
                vec![
                    (
                        "0",
                        "Seconds prior to initial reference time (defined in Section 1)",
                    ),
                    ("1-191", "Reserved"),
                    ("192-254", "Reserved for local use"),
                    ("255", "Missing"),
                ],
            ),
            (
                OptArg::L2(3, 2),
                vec![("0", "Clear sky probability"), ("30", "Measurement cost")],
            ),
            (
                OptArg::L2(20, 0),
                vec![
                    ("0", "Universal thermal climate index"),
                    ("1-191", "Reserved"),
                    ("192-254", "Reserved for local use"),
                    ("255", "Missing"),
                ],
            ),
        ];

        let expected = expected_data
            .iter()
            .map(|(c, table)| {
                (
                    *c,
                    CodeTable {
                        desc: expected_title.clone(),
                        data: table
                            .iter()
                            .map(|(a, b)| (a.to_string(), b.to_string()))
                            .collect::<Vec<_>>(),
                    },
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(tables, expected);
    }

    #[test]
    fn export() {
        let mut db = CodeDB::new();
        db.load(PATH_STR_0).unwrap();
        assert_eq!(
            db.export((0, 0, OptArg::None)),
            "\
/// Foo
const CODE_TABLE_0_0: &[& str] = &[
    \"0A\",
    \"0B\",
];"
        );
    }

    #[test]
    fn format() {
        let mut db = CodeDB::new();
        db.load(PATH_STR_0).unwrap();
        db.load(PATH_STR_1).unwrap();
        db.load(PATH_STR_2).unwrap();
        db.load(PATH_STR_3).unwrap();
        assert_eq!(
            format!("{db}"),
            "\
/// Foo
const CODE_TABLE_0_0: &[& str] = &[
    \"0A\",
    \"0B\",
];

/// Bar
const CODE_TABLE_1_0: &[& str] = &[
    \"1A\",
    \"1B\",
];

/// Baz
const CODE_TABLE_4_1_0: &[& str] = &[
    \"Temperature\",
    \"\",
    \"\",
    \"Mass\",
];

/// Baz
const CODE_TABLE_4_1_3: &[& str] = &[
    \"Image format products\",
    \"Quantitative products\",
    \"Cloud properties\",
];

/// Baz
const CODE_TABLE_4_1_20: &[& str] = &[
    \"Health indicators\",
];

/// Baz
const CODE_TABLE_4_2_0_0: &[& str] = &[
    \"Temperature\",
    \"\",
    \"\",
    \"\",
    \"\",
    \"\",
    \"\",
    \"\",
    \"\",
    \"\",
    \"Latent heat net flux\",
];

/// Baz
const CODE_TABLE_4_2_0_191: &[& str] = &[
    \"Seconds prior to initial reference time (defined in Section 1)\",
];

/// Baz
const CODE_TABLE_4_2_3_2: &[& str] = &[];

/// Baz
const CODE_TABLE_4_2_20_0: &[& str] = &[
    \"Universal thermal climate index\",
];"
        );
    }

    #[test]
    fn codetable_to_vec() {
        let mut db = CodeDB::new();
        db.load(PATH_STR_0).unwrap();
        assert_eq!(
            db.get((0, 0, OptArg::None)).unwrap().to_vec(),
            vec!["0A", "0B",]
        );
    }
}
