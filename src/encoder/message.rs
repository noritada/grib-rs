mod multigrid;
mod multiproduct;
mod single;

pub use multigrid::*;
pub use multiproduct::*;
pub use single::*;

use crate::{Encoder, WriteGrib2ProductDef, WriteGrib2SubmessageL3};

impl<'a, 'd, P> WriteGrib2SubmessageL3 for (&'a P, &'a Encoder<'d>)
where
    P: WriteGrib2ProductDef,
{
    type S4<'s>
        = P
    where
        Self: 's;

    type SD<'s>
        = Encoder<'d>
    where
        Self: 's;

    fn section4(&self) -> &Self::S4<'_> {
        self.0
    }

    fn data_sections(&self) -> &Self::SD<'_> {
        self.1
    }
}
