use std::{borrow::Cow, str::FromStr};

use proc_macro2::Span;
use quote::quote;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Wgrib2Table(Vec<Wgrib2TableEntry>);

impl Wgrib2Table {
    pub(crate) fn from_file<P>(path: P) -> Result<Self, &'static str>
    where
        P: AsRef<std::path::Path>,
    {
        let s = std::fs::read_to_string(path).map_err(|_| "reading file failed")?;
        s.parse::<Self>()
    }

    fn from_str_impl(s: &str) -> Option<Self> {
        let lines: Option<Vec<_>> = s
            .lines()
            .map(|line| line.trim_end().parse::<Wgrib2TableEntry>().ok())
            .collect();
        Some(Self(lines?))
    }

    pub(crate) fn enum_variants(&self) -> proc_macro2::TokenStream {
        let variant_idents = self.entries().map(|ent| ent.enum_variant());

        quote! {
            #(#variant_idents),*
        }
    }

    fn entries(&self) -> std::slice::Iter<Wgrib2TableEntry> {
        let Self(entries) = self;
        entries.iter()
    }
}

impl FromStr for Wgrib2Table {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_impl(s).ok_or("parsing as a wgrib2 table failed")
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Wgrib2TableEntry {
    discipline: u8,
    master_table_start_ver: u8,
    master_table_end_ver: u8,
    centre: u16,
    local_table_ver: u8,
    param_category: u8,
    param_number: u8,
    name: String,
    desc: String,
    unit: String,
}

impl Wgrib2TableEntry {
    fn from_str_impl(s: &str) -> Option<Self> {
        let mut words = s.split(":");
        let discipline = words.next()?.parse::<u8>().ok()?;
        let _ = words.next()?;
        let master_table_start_ver = words.next()?.parse::<u8>().ok()?;
        let master_table_end_ver = words.next()?.parse::<u8>().ok()?;
        let centre = words.next()?.parse::<u16>().ok()?;
        let local_table_ver = words.next()?.parse::<u8>().ok()?;
        let param_category = words.next()?.parse::<u8>().ok()?;
        let param_number = words.next()?.parse::<u8>().ok()?;
        let name = words.next()?.to_owned();
        let desc = words.next()?.to_owned();
        let unit = words.next()?.to_owned();
        Some(Self {
            discipline,
            master_table_start_ver,
            master_table_end_ver,
            centre,
            local_table_ver,
            param_category,
            param_number,
            name,
            desc,
            unit,
        })
    }

    pub(crate) fn enum_variant(&self) -> proc_macro2::TokenStream {
        let name = normalize_name(&self.name);
        let ident = proc_macro2::Ident::new(&name, Span::call_site());
        let num = ((self.discipline as u32) << 16)
            + ((self.param_category as u32) << 8)
            + self.param_number as u32;
        let num = proc_macro2::Literal::u32_unsuffixed(num);
        let doc = format!(
            "Code `{}`. {}. Units: {}.

(Product Discipline {}, Parameter Category {}, Parameter Number {}.)",
            self.name,
            self.desc,
            self.unit,
            self.discipline,
            self.param_category,
            self.param_number
        );
        let doc = proc_macro2::Literal::string(&doc);

        quote! {
            #[doc = #doc]
            #ident = #num
        }
    }
}

impl FromStr for Wgrib2TableEntry {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_impl(s).ok_or("parsing as a wgrib2 table failed")
    }
}

// Makes the specified string available as an Rust enum variant identifier.
// Only supports cases used in the NCEP table.
fn normalize_name(name: &str) -> Cow<str> {
    let name = normalize_name_starting_with_number(name);
    normalize_name_with_hyphens(name)
}

fn normalize_name_starting_with_number(name: &str) -> Cow<str> {
    let mut chars = name.chars();
    match chars.next() {
        Some('4') => {
            let mut s = String::with_capacity(name.len() + 4);
            s.push_str("Four");
            s.push_str(chars.as_str());
            Cow::Owned(s)
        }
        Some('5') => {
            let mut s = String::with_capacity(name.len() + 4);
            s.push_str("Five");
            s.push_str(chars.as_str());
            Cow::Owned(s)
        }
        _ => Cow::Borrowed(name),
    }
}

fn normalize_name_with_hyphens(name: Cow<str>) -> Cow<str> {
    if name.contains('-') {
        let s = name.replace('-', "_");
        Cow::Owned(s)
    } else {
        name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_wgrib2_table() {
        let input = "\
0:1:0:255:0:0:0:0:TMP:Temperature:K
0:1:0:255:0:0:0:1:VTMP:Virtual Temperature:K
";
        let actual = input.parse::<Wgrib2Table>();
        let expected = vec![
            Wgrib2TableEntry {
                discipline: 0,
                master_table_start_ver: 0,
                master_table_end_ver: 255,
                centre: 0,
                local_table_ver: 0,
                param_category: 0,
                param_number: 0,
                name: "TMP".to_owned(),
                desc: "Temperature".to_owned(),
                unit: "K".to_owned(),
            },
            Wgrib2TableEntry {
                discipline: 0,
                master_table_start_ver: 0,
                master_table_end_ver: 255,
                centre: 0,
                local_table_ver: 0,
                param_category: 0,
                param_number: 1,
                name: "VTMP".to_owned(),
                desc: "Virtual Temperature".to_owned(),
                unit: "K".to_owned(),
            },
        ];
        let expected = Ok(Wgrib2Table(expected));
        assert_eq!(actual, expected);
    }
}
