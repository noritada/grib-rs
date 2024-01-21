#[cfg(test)]
pub(crate) mod test_helpers {
    macro_rules! assert_almost_eq {
        ($a1:expr, $a2:expr, $d:expr) => {
            if $a1 - $a2 > $d || $a2 - $a1 > $d {
                panic!();
            }
        };
    }

    macro_rules! test_assert_almost_eq_do_not_panic {
        ($((
            $name:ident,
            $a1:expr,
            $a2:expr,
            $d:expr
        ),)*) => ($(
            #[test]
            fn $name() {
                assert_almost_eq!($a1, $a2, $d)
            }
        )*);
    }

    test_assert_almost_eq_do_not_panic! {
        (assert_almost_eq_does_not_panic_for_positive_lt_positive, 1.01, 1.02, 0.1),
        (assert_almost_eq_does_not_panic_for_positive_gt_positive, 1.02, 1.01, 0.1),
        (assert_almost_eq_does_not_panic_for_negative_lt_negative, -1.02, -1.01, 0.1),
        (assert_almost_eq_does_not_panic_for_negative_gt_negative, -1.01, -1.02, 0.1),
        (assert_almost_eq_does_not_panic_for_positive_negative, 0.01, -0.01, 0.1),
        (assert_almost_eq_does_not_panic_for_negative_positive, -0.01, 0.01, 0.1),
    }

    macro_rules! test_assert_almost_eq_panic {
        ($((
            $name:ident,
            $a1:expr,
            $a2:expr,
            $d:expr
        ),)*) => ($(
            #[test]
            #[should_panic]
            fn $name() {
                assert_almost_eq!($a1, $a2, $d)
            }
        )*);
    }

    test_assert_almost_eq_panic! {
        (assert_almost_eq_panics_for_positive_lt_positive, 1.01, 1.02, 0.001),
        (assert_almost_eq_panics_for_positive_gt_positive, 1.02, 1.01, 0.001),
        (assert_almost_eq_panics_for_negative_lt_negative, -1.02, -1.01, 0.001),
        (assert_almost_eq_panics_for_negative_gt_negative, -1.01, -1.02, 0.001),
        (assert_almost_eq_panics_for_positive_negative, 0.01, -0.01, 0.001),
        (assert_almost_eq_panics_for_negative_positive, -0.01, 0.01, 0.001),
    }

    pub(crate) fn assert_coord_almost_eq((x1, y1): (f32, f32), (x2, y2): (f32, f32), delta: f32) {
        assert_almost_eq!(x1, x2, delta);
        assert_almost_eq!(y1, y2, delta);
    }
}
