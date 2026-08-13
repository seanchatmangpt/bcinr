# CMCA-101: Dwell-time hysteresis mode-switch signal in allocator::allocate() is computed but never wired to output

**Type:** Tech Debt
**Priority:** Medium
**Status:** **Done** — Path (A) implemented: `crates/bcinr-cmca/tests/dwell_time_hysteresis.rs`
(2 tests, both passing). Correction to this ticket's own premise, found while
implementing: the machinery is not fully inert — `switch_wanted`/`can_switch` gate
`update_allowed`, which gates the MWU weight update that feeds `pi_res` on
subsequent calls. It's dead only in the sense this ticket describes: no real
caller anywhere threaded state across calls to exercise it. Interaction found with
CMCA-107: the test's original flat star-shaped tree (root + 7 leaf children) hit
CMCA-107's divergence-guard degenerate case (`kappa_v` identically zero when a
node's children are all leaves), so the test tree was changed to a genuine
two-level shape. See the test file's own doc comments for both findings.

## Summary

allocator::allocate()/allocate_in() computes a dom_mode/prev_mode dwell-time hysteresis signal (last_switch_t, tau_d, switch_wanted, can_switch) on every call, but no code path anywhere in the workspace threads state across successive t values, so the signal never influences the returned pi_res/flow. The dwell-time-lock mode-switching protocol is spec'd in ORIGINAL_REQUEST.md:1355 and stability_proof_draft.md:101-127, but has no exercised implementation.

## Context

crates/bcinr-cmca/src/allocator/mod.rs's allocate()/allocate_in() computes a dom_mode/prev_mode dwell-time hysteresis signal every call: last_switch_t, tau_d, switch_wanted, and can_switch are all derived, but none of them feed back into the pi_res/flow the function returns. The gating machinery is dead in the sense that it runs, produces values, and those values are discarded before affecting output.

This was confirmed by exhaustive grep across both bcinr and mfw: zero real callers exist that thread last_switch_t/prev_mode/weights forward across successive t values. Every test and example either calls allocate() once, or resets state fresh on each call — none replay a t-sequence through a stateful caller loop, which is the only call shape that could exercise the hysteresis logic.

The intended protocol is documented, not invented for this ticket: ORIGINAL_REQUEST.md:1355 and stability_proof_draft.md:101-127 describe a dwell-time-lock mode-switching mechanism, where a caller repeatedly invokes allocate() across successive t, threading last_switch_t/prev_mode/weights state forward call-to-call, and dom_mode is supposed to gate whether a mode switch is actually taken at each step.

This is orthogonal to the newly-added allocate_single_lens() (added this session), which answers a different question — which lens's answer applies for a single call — not how mode-switching hysteresis behaves across a sequence of calls. Do not conflate the two when scoping the fix.

Net effect: the hysteresis computation is real code, compiles, runs on every call, and is entirely inert. This is not a correctness bug today (nothing currently depends on it switching modes), but it is a dead-code / spec-drift risk: the code and the spec both claim mode-switching hysteresis exists as a working mechanism, and neither is backed by an executed proof that it does anything.

## Acceptance Criteria

- [ ] Decision recorded: either (A) build a real stepped-caller integration test/example, or (B) deprecate/remove the dwell-time machinery. No third outcome (leaving it as-is, silently) closes this ticket.
- [ ] If (A) build: a new integration test or example in crates/bcinr-cmca (e.g. tests/dwell_time_hysteresis.rs or examples/) drives allocate()/allocate_in() across a sequence of successive t values, explicitly threading last_switch_t/prev_mode/weights forward between calls, matching the protocol in ORIGINAL_REQUEST.md:1355 and stability_proof_draft.md:101-127.
- [ ] If (A) build: the test asserts on real state, not interaction — e.g. it constructs a t-sequence where switch_wanted becomes true and tau_d has elapsed, and asserts dom_mode/pi_res actually changes at the expected step (state-based, Chicago-style assertion), and a second case where dwell time has NOT elapsed and asserts the mode does NOT switch (hysteresis holds).
- [ ] If (A) build: test is added to the crate's normal `cargo test -p bcinr-cmca` path (not gated behind a feature flag that CI skips), and passes in a real run whose output is captured as evidence, not just described.
- [ ] If (B) deprecate: last_switch_t, tau_d, switch_wanted, can_switch, prev_mode/dom_mode gating logic in allocate()/allocate_in() are removed (or explicitly marked #[deprecated] with a doc comment stating no exercised caller exists), and ORIGINAL_REQUEST.md:1355 / stability_proof_draft.md:101-127 are updated or annotated to reflect that the mechanism is not implemented, so spec and code stop disagreeing.
- [ ] If (B) deprecate: `cargo test -p bcinr-cmca` and `make clippy` pass after removal/deprecation, confirming no other code silently depended on the removed fields.
- [ ] Either resolution path is confirmed via a real grep re-run (`grep -rn 'last_switch_t\|switch_wanted\|can_switch\|dom_mode' crates/bcinr-cmca crates/bcinr-mfw 2>/dev/null` or workspace equivalent) showing the new caller (path A) or the absence of orphaned fields (path B), attached as evidence in the closing comment — not asserted from memory.

## Files likely touched

- `/Users/sac/bcinr/crates/bcinr-cmca/src/allocator/mod.rs`
- `/Users/sac/bcinr/crates/bcinr-cmca/tests/dwell_time_hysteresis.rs`
- `/Users/sac/bcinr/crates/bcinr-cmca/examples/`
- `/Users/sac/bcinr/ORIGINAL_REQUEST.md`
- `/Users/sac/bcinr/stability_proof_draft.md`
