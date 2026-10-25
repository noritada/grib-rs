use grib_template_helpers::{DocOverrides, Dump};

#[derive(grib_template_derive::CodeTable, grib_template_derive::Dump)]
#[code_table(file = "tests/tables/code_table.csv", format = wmo_csv)]
#[dump(code_table)]
struct ByteTable(
    /// Default.
    u8,
);

#[derive(grib_template_derive::Dump, grib_template_derive::CodeTable)]
#[code_table(file = "tests/tables/code_table_unitless.csv", format = wmo_csv)]
#[dump(code_table)]
struct WordTable(u16);

#[derive(grib_template_derive::Dump)]
#[dump(flag, code_table)]
struct ManualTable(u8);

impl grib_template_helpers::CodeTable for ManualTable {
    fn code_description(&self) -> Option<&'static str> {
        Some("One")
    }

    fn code_unit(&self) -> Option<&'static str> {
        None
    }
}

#[derive(grib_template_derive::Dump)]
struct Parent {
    /// Foo.
    foo: ByteTable,
    /// Following.
    following: WordTable,
}

macro_rules! test_dump {
    ($($name:ident: $value:expr, $overrides:expr => ($expected:expr, $pos:expr)),* $(,)?) => {
        $(
            #[test]
            fn $name() -> Result<(), Box<dyn std::error::Error>> {
                let mut output = Vec::new();
                let mut pos = 0;
                $value.dump(Some(&"foo".into()), $overrides, &mut pos, &mut output)?;
                assert_eq!(String::from_utf8(output)?, $expected);
                assert_eq!(pos, $pos);
                Ok(())
            }
        )*
    };
}

test_dump! {
    description: ByteTable(1), DocOverrides::empty()
        => ("0         foo = 1 [Range description]  // Default.\n", 1),
    no_description: ByteTable(255), DocOverrides::empty()
        => ("0         foo = 255  // Default.\n", 1),
    override_doc: ByteTable(1), DocOverrides::new(vec![("", "Override.")])
        => ("0         foo = 1 [Range description]  // Override.\n", 1),
    wide_value: WordTable(65535), DocOverrides::empty()
        => ("0-1       foo = 65535 [Upper bound]\n", 2),
    manual_with_flag: ManualTable(1), DocOverrides::empty()
        => ("0         foo = 0b00000001 [One]\n", 1),
    parent_fields: Parent { foo: ByteTable(1), following: WordTable(256) }, DocOverrides::empty()
        => ("0         foo.foo = 1 [Range description]  // Foo.\n1-2       foo.following = 256 [Wide range]  // Following.\n", 3),
}
