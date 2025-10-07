use std::{
    borrow::Cow,
    collections::{HashMap, hash_map::Entry},
    fmt,
    str::FromStr,
};

use proc_macro2::Span;
use quote::quote;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Wgrib2Table {
    codes: Vec<Wgrib2TableEntry>,
    remapper: HashMap<u32, u32>,
}

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
        let codes = lines?;

        let mut table = HashMap::<String, u32>::with_capacity(codes.len());
        let mut remapper = HashMap::<u32, u32>::with_capacity(codes.len() / 10);
        for code in codes.iter() {
            let key = code.normalized_name();
            let value = code.id();
            match table.entry(key.to_string()) {
                Entry::Occupied(first_value) => {
                    remapper.insert(value, *first_value.get());
                }
                Entry::Vacant(entry) => {
                    entry.insert(value);
                }
            }
        }

        Some(Self { codes, remapper })
    }

    pub(crate) fn enum_variants(&self) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
        let mut merger = HashMap::<u32, Vec<&Wgrib2TableEntry>>::with_capacity(self.remapper.len());
        for entry in self.codes.iter() {
            if let Some(first_code) = self.remapper.get(&entry.id()) {
                merger
                    .entry(*first_code)
                    .and_modify(|list| list.push(entry))
                    .or_insert(vec![entry]);
            }
        }

        let entries = self
            .codes
            .iter()
            .filter(|entry| !self.remapper.contains_key(&entry.id()));
        let variant_idents =
            entries.map(|ent| ent.enum_variant(merger.get(&ent.id()).unwrap_or(&vec![])));
        let mapper = self.codes.iter().map(|entry| {
            let id = entry.id();
            let name = entry.enum_variant_ident();
            quote! { #id => Ok(Self::#name) }
        });

        (
            quote! {
                #(#variant_idents),*
            },
            quote! {
                #(#mapper),*
            },
        )
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

    pub(crate) fn enum_variant(&self, others: &[&Self]) -> proc_macro2::TokenStream {
        let ident = self.enum_variant_ident();
        let num = proc_macro2::Literal::u32_unsuffixed(self.id());
        let mut table_entries = vec![self];
        table_entries.extend_from_slice(others);
        let doc = format!(
            "Code `{}`.

| Product Discipline | Parameter Category | Parameter Number | Description | Units | Abbreviation |
|---|---|---|---|---|---|
{}",
            self.name,
            DocTableEntries(&table_entries)
        );
        let doc = proc_macro2::Literal::string(&doc);

        quote! {
            #[doc = #doc]
            #ident = #num
        }
    }

    pub(crate) fn enum_variant_ident(&self) -> proc_macro2::TokenStream {
        let name = self.normalized_name();
        let ident = proc_macro2::Ident::new(&name, Span::call_site());
        quote! { #ident }
    }

    fn normalized_name(&self) -> String {
        normalize_name(&self.name)
    }

    fn id(&self) -> u32 {
        ((self.discipline as u32) << 16)
            + ((self.param_category as u32) << 8)
            + self.param_number as u32
    }
}

impl FromStr for Wgrib2TableEntry {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_impl(s).ok_or("parsing as a wgrib2 table failed")
    }
}

struct DocTableEntries<'a>(&'a [&'a Wgrib2TableEntry]);

impl fmt::Display for DocTableEntries<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(inner) = self;
        for entry in inner.iter() {
            f.write_fmt(format_args!(
                "| {} | {} | {} | {} | {} | {} |\n",
                entry.discipline,
                entry.param_category,
                entry.param_number,
                entry.desc,
                entry.unit,
                entry.name,
            ))?;
        }
        Ok(())
    }
}

// Makes the specified string available as an Rust enum variant identifier.
fn normalize_name(name: &str) -> String {
    let name = if name.contains('-') {
        let s = name.replace('-', "_");
        Cow::Owned(s)
    } else {
        Cow::Borrowed(name)
    };
    format!("_{name}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_wgrib2_table() {
        let input = "\
0:1:0:255:0:0:0:0:TMP:Temperature:K
0:1:0:255:0:0:0:1:VTMP:Virtual Temperature:K
0:1:0:255:0:0:3:16:U-GWD:Zonal Flux of Gravity Wave Stress:N/m^2
0:0:0:255:7:1:3:194:U-GWD:Zonal Flux of Gravity Wave Stress:N/m^2
";
        let actual = input.parse::<Wgrib2Table>();
        let expected_codes = vec![
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
            Wgrib2TableEntry {
                discipline: 0,
                master_table_start_ver: 0,
                master_table_end_ver: 255,
                centre: 0,
                local_table_ver: 0,
                param_category: 3,
                param_number: 16,
                name: "U-GWD".to_owned(),
                desc: "Zonal Flux of Gravity Wave Stress".to_owned(),
                unit: "N/m^2".to_owned(),
            },
            Wgrib2TableEntry {
                discipline: 0,
                master_table_start_ver: 0,
                master_table_end_ver: 255,
                centre: 7,
                local_table_ver: 1,
                param_category: 3,
                param_number: 194,
                name: "U-GWD".to_owned(),
                desc: "Zonal Flux of Gravity Wave Stress".to_owned(),
                unit: "N/m^2".to_owned(),
            },
        ];
        let expected_remapper = [(0x_00_03_c2, 0x_00_03_10)];
        let expected_remapper = HashMap::from(expected_remapper);
        let expected = Ok(Wgrib2Table {
            codes: expected_codes,
            remapper: expected_remapper,
        });
        assert_eq!(actual, expected);
    }
}
