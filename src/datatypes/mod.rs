mod product_attributes;
pub use product_attributes::*;
mod sections;
pub use sections::*;

pub type MessageIndex = (usize, usize);

pub(crate) struct Grib2SubmessageIndex(
    pub(crate) usize,
    pub(crate) usize,
    pub(crate) Option<usize>,
    pub(crate) usize,
    pub(crate) usize,
    pub(crate) usize,
    pub(crate) usize,
    pub(crate) usize,
    pub(crate) usize,
    MessageIndex,
);

impl Grib2SubmessageIndex {
    pub(crate) fn new(
        message_index: MessageIndex,
        sect0_index: usize,
        sect1_index: usize,
        sect2_index: Option<usize>,
        sect3_index: usize,
        sect4_index: usize,
        sect5_index: usize,
        sect6_index: usize,
        sect7_index: usize,
        sect8_index: usize,
    ) -> Self {
        Self(
            sect0_index,
            sect1_index,
            sect2_index,
            sect3_index,
            sect4_index,
            sect5_index,
            sect6_index,
            sect7_index,
            sect8_index,
            message_index,
        )
    }

    pub(crate) fn message_index(&self) -> MessageIndex {
        self.9
    }
}
