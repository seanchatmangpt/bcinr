use crate::fixed::Fixed;
use crate::allocator::{const_lt_u32, const_select_u32, const_eq_u32};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ObservatoryFlag {
    NumericallyUncertain,
    GramDegenerate,
    Drifting,
    ScaleInert,
    RecertificationCandidate,
}

impl ObservatoryFlag {
    pub fn from_u32(val: u32) -> Option<Self> {
        let lookup = [
            Some(Self::NumericallyUncertain),
            Some(Self::GramDegenerate),
            Some(Self::Drifting),
            Some(Self::ScaleInert),
            Some(Self::RecertificationCandidate),
            None, None, None
        ];
        
        let in_bounds = const_lt_u32(val, 5);
        let idx = const_select_u32(in_bounds, val, 5) as usize;
        let res = lookup[idx & 7];
        
        res.filter(|_| in_bounds != 0)
    }
}

const FLAGS: [ObservatoryFlag; 8] = [
    ObservatoryFlag::NumericallyUncertain,
    ObservatoryFlag::GramDegenerate,
    ObservatoryFlag::Drifting,
    ObservatoryFlag::ScaleInert,
    ObservatoryFlag::RecertificationCandidate,
    ObservatoryFlag::RecertificationCandidate,
    ObservatoryFlag::RecertificationCandidate,
    ObservatoryFlag::RecertificationCandidate,
];

/// Wrap observatory result branchlessly.
///
/// # Branchless Contract
pub fn wrap_observatory_result(
    flag_code: u32,
) -> Result<(), ObservatoryFlag> {
    let flag = FLAGS[(flag_code as usize) & 7];
    let is_recert = const_eq_u32(flag_code, 4);
    let outcomes = [Err(flag), Ok(())];
    outcomes[is_recert as usize]
}

/// Evaluates calibration metrics and proposes an admission flag branchlessly.
///
/// # Branchless Contract
pub fn evaluate_calibration(
    kappa_hat: Fixed,
    kappa_under: Fixed,
    epsilon_on: Fixed,
    _gamma_min_plus_hat: Fixed,
    gamma_min_plus_under: Fixed,
    epsilon_gram: Fixed,
    d_js: Fixed,
    epsilon_drift: Fixed,
    s_meas: Fixed,
    s_leaf: Fixed,
) -> Result<(), ObservatoryFlag> {
    
    // Conditions
    let is_drift = const_lt_u32(epsilon_drift.0, d_js.0);
    
    let is_scale_inert = const_eq_u32(s_meas.0, s_leaf.0);
    
    let kappa_hat_on = const_lt_u32(epsilon_on.0, kappa_hat.0) | const_eq_u32(epsilon_on.0, kappa_hat.0);
    let kappa_under_off = const_lt_u32(kappa_under.0, epsilon_on.0);
    let is_numerically_uncertain = kappa_hat_on & kappa_under_off;
    
    let kappa_under_on = const_lt_u32(epsilon_on.0, kappa_under.0) | const_eq_u32(epsilon_on.0, kappa_under.0);
    
    let gamma_under_off = const_lt_u32(gamma_min_plus_under.0, epsilon_gram.0);
    
    let is_gram_degenerate = kappa_under_on & gamma_under_off;
    
    let is_recert = kappa_under_on & (!gamma_under_off);
    
    let mut flag = 4u32; // Default to Ok
    flag = const_select_u32(is_recert, 4, flag);
    flag = const_select_u32(is_gram_degenerate, 1, flag);
    flag = const_select_u32(is_numerically_uncertain, 0, flag);
    flag = const_select_u32(is_scale_inert, 3, flag);
    flag = const_select_u32(is_drift, 2, flag);
    
    wrap_observatory_result(flag)
}
