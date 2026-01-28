use crate::def::grib2::template::param_set::SimplePacking;

pub fn determine_simple_packing_params(
    values: &[f64],
    dec: Option<i16>,
) -> (SimplePacking, Vec<f64>) {
    let dec = dec.unwrap_or(0);
    let mut min = 0.;
    let mut max = 0.;
    let scaled = values
        .iter()
        .enumerate()
        .map(|(i, value)| {
            let scaled = value * 10_f64.powf(dec as f64);
            (min, max) = if i == 0 {
                (scaled, scaled)
            } else {
                (scaled.min(min), scaled.max(max))
            };
            scaled
        })
        .collect::<Vec<_>>();
    let ref_val = min as f32;
    let range = max - min;
    let exp = 0;
    let num_bits = (range + 1.).log2().ceil() as u8;
    // TODO: if `num_bits` is too large, increase `exp` to reduce `num_bits`.
    let params = SimplePacking {
        ref_val,
        exp,
        dec,
        num_bits,
    };
    (params, scaled)
}
