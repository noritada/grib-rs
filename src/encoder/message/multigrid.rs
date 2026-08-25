use std::iter::{Once, once};

use super::{
    GpvEncoder, WriteGrib2GridDef, WriteGrib2Ident, WriteGrib2LocalUse, WriteGrib2Message,
    WriteGrib2MessageIterL1, WriteGrib2MessageIterL2, WriteGrib2ProductDef,
};

/// A writer for GRIB2 messages containing multiple submessages with different
/// grids.
pub struct MultiGridGrib2Message<'d, I, L, G, P> {
    discipline: u8,
    ident: I,
    local_use: Option<L>,
    grid_product_values: Vec<(G, P, GpvEncoder<'d>)>,
}

impl<'d, I, L, G, P> MultiGridGrib2Message<'d, I, L, G, P> {
    pub fn new(
        discipline: u8,
        ident: I,
        local_use: Option<L>,
        grid_product_values: Vec<(G, P, GpvEncoder<'d>)>,
    ) -> Self {
        Self {
            discipline,
            ident,
            local_use,
            grid_product_values,
        }
    }
}

impl<'d, I, L, G, P> WriteGrib2Message for MultiGridGrib2Message<'d, I, L, G, P>
where
    I: WriteGrib2Ident,
    L: WriteGrib2LocalUse,
    G: WriteGrib2GridDef,
    P: WriteGrib2ProductDef,
{
    type S1<'a>
        = I
    where
        Self: 'a;

    type Item<'a>
        = (&'a Option<L>, &'a Vec<(G, P, GpvEncoder<'d>)>)
    where
        Self: 'a;

    type Iter<'a>
        = Once<(&'a Option<L>, &'a Vec<(G, P, GpvEncoder<'d>)>)>
    where
        Self: 'a;

    fn discipline(&self) -> u8 {
        self.discipline
    }

    fn section1(&self) -> &Self::S1<'_> {
        &self.ident
    }

    fn iter(&self) -> Self::Iter<'_> {
        once((&self.local_use, &self.grid_product_values))
    }
}

impl<'a, 'd, L, G, P> WriteGrib2MessageIterL1 for (&'a Option<L>, &'a Vec<(G, P, GpvEncoder<'d>)>)
where
    L: WriteGrib2LocalUse,
    G: WriteGrib2GridDef,
    P: WriteGrib2ProductDef,
{
    type S2<'s>
        = L
    where
        Self: 's;

    type Item<'s>
        = &'a (G, P, GpvEncoder<'d>)
    where
        Self: 's;

    type Iter<'s>
        = std::slice::Iter<'a, (G, P, GpvEncoder<'d>)>
    where
        Self: 's;

    fn section2(&self) -> Option<&Self::S2<'_>> {
        self.0.as_ref()
    }

    fn iter(&self) -> Self::Iter<'_> {
        self.1.iter()
    }
}

impl<'d, G, P> WriteGrib2MessageIterL2 for (G, P, GpvEncoder<'d>)
where
    G: WriteGrib2GridDef,
    P: WriteGrib2ProductDef,
{
    type S3<'s>
        = G
    where
        Self: 's;

    type Item<'s>
        = (&'s P, &'s GpvEncoder<'d>)
    where
        Self: 's;

    type Iter<'s>
        = Once<(&'s P, &'s GpvEncoder<'d>)>
    where
        Self: 's;

    fn section3(&self) -> &Self::S3<'_> {
        &self.0
    }

    fn iter(&self) -> Self::Iter<'_> {
        once((&self.1, &self.2))
    }
}
