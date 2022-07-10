mod product_attributes;
pub use product_attributes::*;
mod sections;
pub use sections::*;

pub(crate) struct Grib2SubmessageIndex {
    pub(crate) message: usize,
    #[allow(dead_code)] // tentative
    pub(crate) submessage: usize,
    pub(crate) sections: (
        usize,
        usize,
        Option<usize>,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
    ),
}
