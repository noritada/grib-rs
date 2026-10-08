use grib_template_helpers::{DocOverrides, Dump};

#[derive(grib_template_derive::Dump)]
struct Item {
    /// Value
    value: u16,
    flag: u8,
}

#[derive(grib_template_derive::Dump)]
struct Params {
    #[dump(doc(value = "Element value"))]
    items: Vec<Item>,
    optional: Option<Vec<Item>>,
    empty: Vec<Item>,
    absent: Option<Vec<Item>>,
    numbers: Vec<i16>,
    tail: u8,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let params = Params {
        items: vec![Item { value: 10, flag: 1 }, Item { value: 20, flag: 2 }],
        optional: Some(vec![Item { value: 30, flag: 3 }]),
        empty: vec![],
        absent: None,
        numbers: vec![-1, 2],
        tail: 9,
    };
    let mut output = Vec::new();
    let mut pos = 1;
    params.dump(
        Some(&"root".into()),
        DocOverrides::new(vec![("items.flag", "Element flag")]),
        &mut pos,
        &mut output,
    )?;
    assert_eq!(
        String::from_utf8(output)?,
        "\
1-2       root.items[0].value = 10  // Element value
3         root.items[0].flag = 1  // Element flag
4-5       root.items[1].value = 20  // Element value
6         root.items[1].flag = 2  // Element flag
7-8       root.optional[0].value = 30  // Value
9         root.optional[0].flag = 3
10-13     root.numbers = [-1, 2]
14        root.tail = 9
"
    );
    assert_eq!(pos, 15);

    let mut output = Vec::new();
    let mut pos = 1;
    params
        .items
        .dump(None, DocOverrides::empty(), &mut pos, &mut output)?;
    assert_eq!(
        String::from_utf8(output)?,
        "\
1-2       [0].value = 10  // Value
3         [0].flag = 1
4-5       [1].value = 20  // Value
6         [1].flag = 2
"
    );
    assert_eq!(pos, 7);
    Ok(())
}
