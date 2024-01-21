#[cfg(test)]
pub(crate) mod test_helpers {
    macro_rules! assert_almost_eq {
        ($a1:expr, $a2:expr, $d:expr) => {
            if $a1 - $a2 > $d || $a2 - $a1 > $d {
                panic!();
            }
        };
    }

    pub(crate) fn assert_coord_almost_eq((x1, y1): (f32, f32), (x2, y2): (f32, f32), delta: f32) {
        assert_almost_eq!(x1, x2, delta);
        assert_almost_eq!(y1, y2, delta);
    }
}
