pub(crate) fn m(sin_phi: f64, cos_phi: f64, e_sq: f64) -> f64 {
    cos_phi / (1. - e_sq * sin_phi * sin_phi).sqrt()
}

// See formula deformation for pj_tsfn().
pub(crate) fn t(cos_phi: f64, sin_phi: f64, e: f64) -> f64 {
    (e * (e * sin_phi).atanh()).exp()
        * if sin_phi > 0. {
            cos_phi / (1. + sin_phi)
        } else {
            (1. - sin_phi) / cos_phi
        }
}

// See formula deformation for pj_phi2().
pub(crate) fn phi2(ts0: f64, e: f64) -> Option<f64> {
    let phi2 = sinhpsi2tanphi((1. / ts0 - ts0) / 2., e)?.atan();
    Some(phi2)
}

// See formula deformation for pj_sinhpsi2tanphi().
pub(crate) fn sinhpsi2tanphi(taup: f64, e: f64) -> Option<f64> {
    const MAX_ITER: usize = 5;
    let root_eps: f64 = f64::EPSILON.sqrt();
    let tol: f64 = root_eps / 10.; // the criterion for Newton's method
    let tmax: f64 = 2. / root_eps; // threshold for large arg limit exact
    let e2m: f64 = 1. - e * e;
    let stol: f64 = tol * 1.0_f64.max(taup.abs());

    // The initial guess.  70 corresponds to chi = 89.18 deg
    let mut tau = if taup.abs() > 70. {
        taup * (e * e.atanh()).exp()
    } else {
        taup / e2m
    };

    // handles +/-inf and nan and e = 1
    if tau.abs() >= tmax {
        return Some(tau);
    }

    let mut count = MAX_ITER;
    while count > 0 {
        let tau1 = (1. + tau * tau).sqrt();
        let sig = (e * (e * tau / tau1).atanh()).sinh();
        let taupa = (1. + sig * sig).sqrt() * tau - sig * tau1;
        let dtau =
            (taup - taupa) * (1. + e2m * (tau * tau)) / (e2m * tau1 * (1. + taupa * taupa).sqrt());

        tau += dtau;

        // backwards test to allow nans to succeed.
        if dtau.abs() < stol {
            return Some(tau);
        }

        count -= 1;
    }
    None
}
