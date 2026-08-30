#[inline]
pub(crate) fn num_octets<N>(num_bits: N) -> usize
where
    usize: From<N>,
{
    usize::from(num_bits).div_ceil(8)
}
