use grib_template_helpers::{CodeTable, DocOverrides, Dump};

use crate::{
    Grib2Context, ParameterContext,
    def::grib2::{
        table::{ParamCategory, ParamNum},
        template::param_set::ProductParam,
    },
};

macro_rules! test_lookup {
    ($($name:ident: $table:expr, $context:expr => ($description:expr, $unit:expr)),* $(,)?) => {
        $(
            #[test]
            fn $name() {
                assert_eq!($table.code_description(&$context), $description);
                assert_eq!($table.code_unit(&$context), $unit);
            }
        )*
    };
}

test_lookup! {
    meteorological_category: ParamCategory(0), Grib2Context { discipline: 0 } => (Some("Temperature"), None),
    hydrological_category: ParamCategory(0), Grib2Context { discipline: 1 } => (Some("Hydrology basic products"), None),
    unknown_discipline: ParamCategory(0), Grib2Context { discipline: 255 } => (None, None),
    local_category: ParamCategory(193), Grib2Context { discipline: 0 } => (None, None),
    temperature: ParamNum(0), ParameterContext { discipline: 0, category: 0 } => (Some("Temperature"), Some("K")),
    hydrological_parameter: ParamNum(0), ParameterContext { discipline: 1, category: 0 } => (Some("Flash flood guidance (Encoded as an accumulation over a floating subinterval of time between the reference time and valid time)"), Some("kg m-2")),
    unknown_parameter_discipline: ParamNum(0), ParameterContext { discipline: 255, category: 0 } => (None, None),
    moisture: ParamNum(0), ParameterContext { discipline: 0, category: 1 } => (Some("Specific humidity"), Some("kg/kg")),
    unknown_category: ParamNum(0), ParameterContext { discipline: 0, category: 193 } => (None, None),
    missing_parameter: ParamNum(255), ParameterContext { discipline: 0, category: 0 } => (None, None),
}

#[test]
fn product_dump_supplements_context() -> Result<(), Box<dyn std::error::Error>> {
    let product = ProductParam {
        category: ParamCategory(1),
        num: ParamNum(0),
    };
    let mut output = Vec::new();
    product.dump(
        &Grib2Context { discipline: 0 },
        None,
        DocOverrides::empty(),
        &mut 1,
        &mut output,
    )?;
    let output = String::from_utf8(output)?;
    assert!(output.contains("category = 1 [Moisture]"));
    assert!(output.contains("num = 0 [Specific humidity]"));
    Ok(())
}
