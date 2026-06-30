//! BLAKE3 chain verification — mirrors `chain_witnesses()` in bcinr-pddl/src/llm_bridge.rs.
//!
//! Factored here so both `receipt_inspect` (MCP tool) and `generate_compliance_report`
//! share the same verification logic without a cross-crate dependency on bcinr-pddl.

/// Recompute `BLAKE3(domain_witness || problem_witness || plan_chain_hash)` and compare
/// against the stored `manufacture_chain` hex string.
///
/// Returns `true` when the receipt is cryptographically intact.
///
/// # Examples
///
/// ```
/// use bcinr_compliance::chain::verify_manufacture_chain;
/// // A fresh chain from three non-empty witnesses always computes deterministically.
/// let valid = verify_manufacture_chain("d_wit", "p_wit", "plan_hash", "");
/// assert!(!valid, "empty stored chain → integrity check fails");
/// ```
pub fn verify_manufacture_chain(
    domain_w: &str,
    problem_w: &str,
    plan_chain: &str,
    stored_chain: &str,
) -> bool {
    if domain_w.is_empty()
        || problem_w.is_empty()
        || plan_chain.is_empty()
        || stored_chain.is_empty()
        || plan_chain == "REFUSED"
    {
        return false;
    }
    let mut h = blake3::Hasher::new();
    h.update(domain_w.as_bytes());
    h.update(problem_w.as_bytes());
    h.update(plan_chain.as_bytes());
    let computed: String = h
        .finalize()
        .as_bytes()
        .iter()
        .map(|x| format!("{x:02x}"))
        .collect();
    computed == stored_chain
}
