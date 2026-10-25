use grib_template_helpers::CodeTable;

#[derive(grib_template_derive::CodeTable)]
#[code_table(file = "tests/tables/code_table.csv", format = wmo_csv)]
struct ByteTable(u8);

#[derive(grib_template_derive::CodeTable)]
#[code_table(format = wmo_csv(), file = "tests/tables/code_table.csv")]
struct WordTable(u16);

#[test]
fn derive_for_u8() {
    let table = ByteTable(0);
    assert_eq!(table.0, 0);
    assert_eq!(table.code_description(), None);
    assert_eq!(table.code_unit(), None);
}

#[test]
fn derive_for_u16() {
    let table = WordTable(256);
    assert_eq!(table.0, 256);
    assert_eq!(table.code_description(), None);
    assert_eq!(table.code_unit(), None);
}
