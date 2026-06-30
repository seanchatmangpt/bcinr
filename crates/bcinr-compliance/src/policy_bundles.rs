//! Named Prolog8 policy bundles for regulatory compliance evaluation.
//!
//! Each bundle is a slice of predicate clause strings.  The compliance engine
//! evaluates each clause against the receipt's materialised fields — no Prolog
//! interpreter is needed; each predicate name maps to a closure over the receipt.

/// GDPR Article 5(1)(c) — data minimisation.
///
/// Ensures: minimal action surface (≤20 steps), no personal-data storage
/// actions, goal reached (purpose limitation).
pub const GDPR_DATA_MINIMISATION: &[&str] = &[
    "step_count_lte(20)",
    "no_action_matching(store_personal.*)",
    "goal_reached",
];

/// HIPAA § 164.312 — PHI handling controls.
///
/// Ensures: admission gate passed, manufacture chain intact (tamper-evident),
/// no uncontrolled PHI transmission actions, all witnesses populated.
pub const HIPAA_PHI_HANDLING: &[&str] = &[
    "admitted",
    "chain_integrity_valid",
    "no_action_matching(transmit.*)",
    "all_witnesses_non_empty",
];

/// MiFID II / SEC Rule 605 — best execution.
///
/// Ensures: goal reached, makespan bounded (≤100 time units), at least one
/// action taken, chain intact, admission gate passed.
pub const FINANCIAL_BEST_EXECUTION: &[&str] = &[
    "goal_reached",
    "makespan_lte(100.0)",
    "step_count_gte(1)",
    "chain_integrity_valid",
    "admitted",
];

/// Look up a named bundle by identifier string.
///
/// Returns `None` for unknown bundle names so callers can surface a clear error.
pub fn lookup(name: &str) -> Option<&'static [&'static str]> {
    match name {
        "gdpr_data_minimisation" => Some(GDPR_DATA_MINIMISATION),
        "hipaa_phi_handling" => Some(HIPAA_PHI_HANDLING),
        "financial_best_execution" => Some(FINANCIAL_BEST_EXECUTION),
        _ => None,
    }
}
