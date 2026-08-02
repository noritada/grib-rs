use std::iter::{Once, once};

use super::{
    Encoder, WriteGrib2GridDef, WriteGrib2Ident, WriteGrib2LocalUse, WriteGrib2Message,
    WriteGrib2MessageIterL1, WriteGrib2MessageIterL2, WriteGrib2MessageIterL3,
    WriteGrib2ProductDef,
};

pub struct MultiProductGrib2Message<'d, I, L, G, P> {
    pub(crate) discipline: u8,
    pub(crate) ident: I,
    pub(crate) local_use: Option<L>,
    pub(crate) grid: G,
    pub(crate) product_values: Vec<(P, Encoder<'d>)>,
}

impl<'d, I, L, G, P> MultiProductGrib2Message<'d, I, L, G, P> {
    pub fn new(
        discipline: u8,
        ident: I,
        local_use: Option<L>,
        grid: G,
        product_values: Vec<(P, Encoder<'d>)>,
    ) -> Self {
        Self {
            discipline,
            ident,
            local_use,
            grid,
            product_values,
        }
    }
}

impl<'d, I, L, G, P> WriteGrib2Message for MultiProductGrib2Message<'d, I, L, G, P>
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
        = (&'a Option<L>, &'a G, &'a Vec<(P, Encoder<'d>)>)
    where
        Self: 'a;

    type Iter<'a>
        = Once<(&'a Option<L>, &'a G, &'a Vec<(P, Encoder<'d>)>)>
    where
        Self: 'a;

    fn discipline(&self) -> u8 {
        self.discipline
    }

    fn section1(&self) -> &Self::S1<'_> {
        &self.ident
    }

    fn iter(&self) -> Self::Iter<'_> {
        once((&self.local_use, &self.grid, &self.product_values))
    }
}

impl<'a, 'd, L, G, P> WriteGrib2MessageIterL1 for (&'a Option<L>, &'a G, &'a Vec<(P, Encoder<'d>)>)
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
        = (&'a G, &'a Vec<(P, Encoder<'d>)>)
    where
        Self: 's;

    type Iter<'s>
        = Once<(&'a G, &'a Vec<(P, Encoder<'d>)>)>
    where
        Self: 's;

    fn section2(&self) -> Option<&Self::S2<'_>> {
        self.0.as_ref()
    }

    fn iter(&self) -> Self::Iter<'_> {
        once((self.1, self.2))
    }
}

impl<'a, 'd, G, P> WriteGrib2MessageIterL2 for (&'a G, &'a Vec<(P, Encoder<'d>)>)
where
    G: WriteGrib2GridDef,
    P: WriteGrib2ProductDef,
{
    type S3<'s>
        = G
    where
        Self: 's;

    type Item<'s>
        = &'a (P, Encoder<'d>)
    where
        Self: 's;

    type Iter<'s>
        = std::slice::Iter<'a, (P, Encoder<'d>)>
    where
        Self: 's;

    fn section3(&self) -> &Self::S3<'_> {
        self.0
    }

    fn iter(&self) -> Self::Iter<'_> {
        self.1.iter()
    }
}

impl<'d, P> WriteGrib2MessageIterL3 for &(P, Encoder<'d>)
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
        &self.0
    }

    fn data_sections(&self) -> &Self::SD<'_> {
        &self.1
    }
}
