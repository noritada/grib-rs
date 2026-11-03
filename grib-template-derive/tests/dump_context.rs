use std::{borrow::Cow, io::Write};

use grib_template_helpers::{DocOverrides, Dump, Dumpable};

struct Context {
    description: &'static str,
}

struct Value(u8);

impl Dumpable for Value {}

impl Dump<Context> for Value {
    fn dump<'d, W: Write>(
        &self,
        context: &Context,
        parent: Option<&Cow<str>>,
        _doc_overrides: DocOverrides<'d>,
        pos: &mut usize,
        output: &mut W,
    ) -> std::io::Result<()> {
        grib_template_helpers::write_position_column(output, pos, 1)?;
        writeln!(
            output,
            "{} = {} [{}]",
            parent.map_or("", |p| p.as_ref()),
            self.0,
            context.description
        )
    }
}

#[derive(grib_template_derive::Dump)]
struct Nested<'a, __DumpContext>
where
    __DumpContext: 'a,
{
    values: Vec<Choice<'a, __DumpContext>>,
    absent: Option<u8>,
    trailing: u8,
}

#[derive(grib_template_derive::Dump)]
enum Choice<'a, T> {
    Value(Inner<'a, T>),
}

#[derive(grib_template_derive::Dump)]
struct Inner<'a, T> {
    value: Option<&'a T>,
}

#[test]
fn context_reaches_nested_values() -> Result<(), Box<dyn std::error::Error>> {
    let value = Value(7);
    let nested = Nested {
        values: vec![
            Choice::Value(Inner {
                value: Some(&value),
            }),
            Choice::Value(Inner { value: None }),
        ],
        absent: None,
        trailing: 9,
    };
    let context = Context {
        description: "external description",
    };
    let mut output = Vec::new();
    let mut pos = 1;
    nested.dump(&context, None, DocOverrides::empty(), &mut pos, &mut output)?;
    assert_eq!(
        String::from_utf8(output)?,
        "1         values[0].value = 7 [external description]\n2         trailing = 9\n",
    );
    assert_eq!(pos, 3);
    Ok(())
}

#[test]
fn unsized_context_is_forwarded() -> Result<(), Box<dyn std::error::Error>> {
    #[derive(grib_template_derive::Dump)]
    struct Scalar {
        value: u8,
    }
    let mut output = Vec::new();
    Scalar { value: 3 }.dump("context", None, DocOverrides::empty(), &mut 1, &mut output)?;
    assert_eq!(String::from_utf8(output)?, "1         value = 3\n");
    Ok(())
}

struct ParameterContext<'a> {
    parent: &'a Context,
    category: u8,
}

impl Context {
    fn with_category(&self, category: u8) -> ParameterContext<'_> {
        ParameterContext {
            parent: self,
            category,
        }
    }
}

impl Dump<ParameterContext<'_>> for Value {
    fn dump<'d, W: Write>(
        &self,
        context: &ParameterContext<'_>,
        parent: Option<&Cow<str>>,
        doc_overrides: DocOverrides<'d>,
        pos: &mut usize,
        output: &mut W,
    ) -> std::io::Result<()> {
        assert_eq!(context.category, 4);
        assert_eq!(doc_overrides.get(""), Some("parameter"));
        self.dump(context.parent, parent, doc_overrides, pos, output)
    }
}

#[derive(grib_template_derive::Dump)]
#[dump(context = Context,)]
struct Product {
    // Put the dependent field first to verify independence from dump order.
    #[dump(
        context = context.with_category(self.category),
        doc(value = "parameter"),
    )]
    number: InnerNumber,
    category: u8,
    unchanged: Value,
}

#[derive(grib_template_derive::Dump)]
struct InnerNumber {
    value: Value,
}

#[derive(grib_template_derive::Dump)]
#[dump(context = Context)]
enum ProductChoice {
    Product(Product),
}

#[test]
fn field_context_is_scoped_to_the_child() -> Result<(), Box<dyn std::error::Error>> {
    let product = ProductChoice::Product(Product {
        number: InnerNumber { value: Value(7) },
        category: 4,
        unchanged: Value(9),
    });
    let mut output = Vec::new();
    let mut pos = 1;
    product.dump(
        &Context {
            description: "external",
        },
        None,
        DocOverrides::empty(),
        &mut pos,
        &mut output,
    )?;
    assert_eq!(
        String::from_utf8(output)?,
        "1         number.value = 7 [external]\n2         category = 4\n3         unchanged = 9 [external]\n",
    );
    assert_eq!(pos, 4);
    Ok(())
}
