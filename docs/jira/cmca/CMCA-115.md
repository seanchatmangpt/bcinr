# CMCA-115: allocation_receipt has no cyclic-parent check and a non-cryptographic digest oversold as tamper-evident

**Type:** Bug / Security-relevant Design Gap
**Priority:** High

## Summary

Three compounding gaps in `crates/bcinr-cmca/src/allocation_receipt.rs`:

1. **No cyclic-parent check.** `allocate_in` refuses on a cyclic `parent`
   (when `proof=Some`) with `StabilityRefusal::ContractViolation`. The
   receipt module's `derive_topology`/`recompute_share` never compute or
   check the cycle witness at all — a cyclic `parent` produces silently
   garbage topology, and the module will happily mint and **verify** a
   receipt for a share that the real, contract-checked allocation function
   would have refused to produce.
2. **Weak digest oversold as "tamper-evident."** `inputs_digest`'s `mix64` is
   a non-cryptographic splitmix64-style finalizer, not a cryptographic hash.
   It is the *only* mechanism binding a receipt to its claimed
   `states`/`parent`/`weights` content. The module's own doc comment uses
   security-property language ("tamper-evident," "confirm the share without
   needing the sealer's trust") that the actual digest strength does not
   support — contrast with this same crate's `bcinr-powl` sibling
   (`OcelCausalReceipt`), which uses real BLAKE3.
3. **Unenforced weights-timing requirement.** The doc comment correctly warns
   that `weights` must be the post-call snapshot, but nothing in
   `AllocationBindings` can detect if a caller passes the wrong (e.g.
   pre-call) snapshot — the receipt will verify internally-consistently
   while asserting a false claim about what the real `allocate` call
   produced.

## Context

Found by adversarial review of the `allocation_receipt` module added this
session.

- Cycle gap: `crates/bcinr-cmca/src/allocation_receipt.rs:132-162`
  (`derive_topology`) and `:207-232` (`recompute_share`) never call
  `check_hierarchy_acyclic` or compute the `P[7][j] != -1` witness
  `allocate_in` uses (`allocator/mod.rs:1546-1549`). `ancestor_doubling_table`
  doesn't panic/hang on a cycle (bounded 8-round doubling) — it silently
  produces garbage `is_descendant`/`is_subtree_leaf` tables, which
  `recompute_share` then happily uses.
- Digest gap: `allocation_receipt.rs:95-104` (`mix64`), `:108-128`
  (`inputs_digest`). Compare `crates/bcinr-powl/src/receipt/causal_receipt.rs:172-193`'s
  real `blake3::Hasher` chaining. Module header
  (`allocation_receipt.rs:1,29-30`) uses "tamper-evident"/"without needing
  the sealer's trust" language not backed by the digest's actual collision
  resistance. Note: this module is internally consistent with
  `certification::seal_certificate`'s own use of the same `mix64` pattern
  (`certification.rs:140`) — it does not claim to inherit BLAKE3-grade
  guarantees, but the module's own stated purpose language reads as a
  stronger security property than the mechanism provides.
- Timing gap: `allocation_receipt.rs:26-28` states the requirement in prose;
  no field in `AllocationBindings` (`:48-54`) or check in
  `seal_allocation_receipt`/`verify_allocation_receipt` can detect a
  pre/post-call mismatch.
- No existing test in the module's `#[cfg(test)] mod tests` (lines 340-542)
  exercises a cyclic `parent` or a pre-call-vs-post-call weights mismatch —
  both gaps are unexercised, not just undocumented.

## Acceptance Criteria

- [ ] Add a cyclic-`parent` check to `seal_allocation_receipt` (and/or
      `verify_allocation_receipt`), refusing with a new
      `AllocationRefusal::Cyclic` variant (mirroring
      `LensSelectionRefusal::Cyclic` from `allocate_single_lens`) rather than
      silently computing over garbage topology.
- [ ] Add a regression test: a cyclic `parent` must refuse at seal time, not
      produce a "verifying" receipt.
- [ ] Decide whether `mix64` is acceptable for this module's actual use case
      (an internal audit trail, not a security boundary) or whether it
      should be upgraded to BLAKE3 (matching `bcinr-powl`'s pattern, which
      this crate could plausibly depend on or vendor a subset of). Either
      way, correct the module's doc-comment language to match the real
      guarantee — "tamper-evident" and "without needing the sealer's trust"
      should not ship unqualified if the mechanism is a 64-bit non-cryptographic
      checksum.
- [ ] Consider adding a lightweight timing/provenance field (e.g. binding the
      receipt to the specific `allocate`/`allocate_in` call's `t`/`digest`
      parameters, if available) to at least narrow the unenforced-timing gap,
      or explicitly document it as a permanent, accepted caller-discipline
      requirement if no cheap fix exists.
- [ ] `cargo test -p bcinr-cmca --features std` full suite green.

## Files likely touched

- `crates/bcinr-cmca/src/allocation_receipt.rs`
