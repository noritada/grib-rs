use grib_template_helpers::CodeTable;

#[derive(grib_template_derive::CodeTable)]
#[code_table(file = "tests/tables/code_table.csv", format = wmo_csv)]
struct ByteTable(u8);

#[derive(grib_template_derive::CodeTable)]
#[code_table(format = wmo_csv(), file = "tests/tables/code_table.csv")]
struct WordTable(u16);

#[derive(grib_template_derive::CodeTable)]
#[code_table(file = "tests/tables/code_table_unitless.csv", format = wmo_csv)]
struct UnitlessTable(u16);

macro_rules! test_lookup {
    ($($name:ident: $table:expr => ($description:expr, $unit:expr)),* $(,)?) => {
        $(
            #[test]
            fn $name() {
                let table = $table;
                let description: Option<&'static str> = table.code_description(&());
                let unit: Option<&'static str> = table.code_unit(&());
                assert_eq!(description, $description);
                assert_eq!(unit, $unit);
            }
        )*
    };
}

test_lookup! {
    escaped_text: ByteTable(0) => (Some("Example, with \"quotes\""), Some("m, \"unit\"")),
    range_start: ByteTable(1) => (Some("Range description"), Some("K")),
    range_middle: ByteTable(3) => (Some("Range description"), Some("K")),
    range_end: ByteTable(5) => (Some("Range description"), Some("K")),
    empty_description: ByteTable(6) => (None, None),
    reserved: ByteTable(7) => (None, None),
    local_use: ByteTable(192) => (None, None),
    missing: ByteTable(255) => (None, None),
    word_with_unit: WordTable(3) => (Some("Range description"), Some("K")),
    uncovered_code: WordTable(256) => (None, None),
    unitless_reserved: UnitlessTable(255) => (None, None),
    wide_range_start: UnitlessTable(256) => (Some("Wide range"), None),
    wide_range_middle: UnitlessTable(1024) => (Some("Wide range"), None),
    wide_range_end: UnitlessTable(65534) => (Some("Wide range"), None),
    u16_max: UnitlessTable(65535) => (Some("Upper bound"), None),
}
