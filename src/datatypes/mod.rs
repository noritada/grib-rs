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
        section_indices: (
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
    ) -> Self {
        Self(
            section_indices.0,
            section_indices.1,
            section_indices.2,
            section_indices.3,
            section_indices.4,
            section_indices.5,
            section_indices.6,
            section_indices.7,
            section_indices.8,
            message_index,
        )
    }

    pub(crate) fn message_index(&self) -> MessageIndex {
        self.9
    }
}
