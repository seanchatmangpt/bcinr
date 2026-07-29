//! Shared BLAKE3 hash-chain fold, used by every receipt kind in this crate
//! that needs the "prior_hash folded with canonical frame bytes" discipline:
//! [`crate::receipt::causal_receipt::OcelCausalReceipt::chain`] (the original, for
//! `OcelCausalFrame`), [`crate::receipt::projection::seal_projection_receipt`],
//! [`crate::receipt::execution::seal_execution_receipt`], and
//! [`crate::receipt::planning::seal_planning_receipt`].
//!
//! `causal_receipt.rs`'s chain predates this module and is left untouched
//! (its own streaming-`Hasher` implementation is not routed through here) —
//! this module exists so the three new receipt kinds share one
//! implementation rather than each hand-rolling the same two-`update()`
//! pattern.

use bcinr_mfw_ir::Digest;

/// Fold `prior_hash` with `canonical_bytes` into a new [`Digest`]:
/// `BLAKE3(prior_hash || canonical_bytes)`. Mirrors
/// `OcelCausalReceipt::chain`'s streaming `Hasher::update(chain_hash)` then
/// `Hasher::update(frame_bytes)` — same two-part fold, generalized over
/// whatever canonical byte buffer a receipt-sealing function built for its
/// own fields (excluding `prior_hash` and `hash` themselves, which are never
/// part of `canonical_bytes` — `prior_hash` is folded in separately here,
/// and `hash` is this function's own output).
pub fn fold(prior_hash: &Digest, canonical_bytes: &[u8]) -> Digest {
    let mut hasher = blake3::Hasher::new();
    hasher.update(prior_hash.as_bytes());
    hasher.update(canonical_bytes);
    Digest::from(*hasher.finalize().as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fold_is_deterministic() {
        let a = fold(&Digest::ZERO, b"payload");
        let b = fold(&Digest::ZERO, b"payload");
        assert_eq!(a, b);
    }

    #[test]
    fn fold_is_sensitive_to_prior_hash() {
        let a = fold(&Digest::ZERO, b"payload");
        let b = fold(&Digest::hash(b"other"), b"payload");
        assert_ne!(a, b);
    }

    #[test]
    fn fold_is_sensitive_to_payload() {
        let a = fold(&Digest::ZERO, b"payload-1");
        let b = fold(&Digest::ZERO, b"payload-2");
        assert_ne!(a, b);
    }
}
