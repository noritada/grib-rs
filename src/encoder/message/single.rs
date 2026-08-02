use std::iter::{Once, once};

use super::{
    Encoder, WriteGrib2GridDef, WriteGrib2Ident, WriteGrib2LocalUse, WriteGrib2Message,
    WriteGrib2ProductDef, WriteGrib2SubmessageL1, WriteGrib2SubmessageL2,
};

pub struct SingleGrib2Message<'d, I, L, G, P> {
    pub(crate) discipline: u8,
    pub(crate) ident: I,
    pub(crate) local_use: Option<L>,
    pub(crate) grid: G,
    pub(crate) product: P,
    pub(crate) values: Encoder<'d>,
}

impl<'d, I, L, G, P> SingleGrib2Message<'d, I, L, G, P> {
    pub fn new(
        discipline: u8,
        ident: I,
        local_use: Option<L>,
        grid: G,
        product: P,
        values: Encoder<'d>,
    ) -> Self {
        Self {
            discipline,
            ident,
            local_use,
            grid,
            product,
            values,
        }
    }
}

impl<'d, I, L, G, P> WriteGrib2Message for SingleGrib2Message<'d, I, L, G, P>
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
        = (&'a Option<L>, &'a G, &'a P, &'a Encoder<'d>)
    where
        Self: 'a;

    type Iter<'a>
        = Once<(&'a Option<L>, &'a G, &'a P, &'a Encoder<'d>)>
    where
        Self: 'a;

    fn discipline(&self) -> u8 {
        self.discipline
    }

    fn section1(&self) -> &Self::S1<'_> {
        &self.ident
    }

    fn iter(&self) -> Self::Iter<'_> {
        once((&self.local_use, &self.grid, &self.product, &self.values))
    }
}

impl<'a, 'd, L, G, P> WriteGrib2SubmessageL1 for (&'a Option<L>, &'a G, &'a P, &'a Encoder<'d>)
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
        = (&'a G, &'a P, &'a Encoder<'d>)
    where
        Self: 's;

    type Iter<'s>
        = Once<(&'a G, &'a P, &'a Encoder<'d>)>
    where
        Self: 's;

    fn section2(&self) -> Option<&Self::S2<'_>> {
        self.0.as_ref()
    }

    fn iter(&self) -> Self::Iter<'_> {
        once((self.1, self.2, self.3))
    }
}

impl<'a, 'd, G, P> WriteGrib2SubmessageL2 for (&'a G, &'a P, &'a Encoder<'d>)
where
    G: WriteGrib2GridDef,
    P: WriteGrib2ProductDef,
{
    type S3<'s>
        = G
    where
        Self: 's;

    type Item<'s>
        = (&'a P, &'a Encoder<'d>)
    where
        Self: 's;

    type Iter<'s>
        = Once<(&'a P, &'a Encoder<'d>)>
    where
        Self: 's;

    fn section3(&self) -> &Self::S3<'_> {
        self.0
    }

    fn iter(&self) -> Self::Iter<'_> {
        once((self.1, self.2))
    }
}
