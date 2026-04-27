pub struct NonStdLenUint<N> {
    val: N,
    num_octets: usize,
}

impl<N> NonStdLenUint<N> {
    pub fn new(val: N, num_octets: usize) -> Self {
        Self { val, num_octets }
    }

    pub fn val(&self) -> &N {
        &self.val
    }

    pub fn num_octets(&self) -> usize {
        self.num_octets
    }
}
