# Refusal Algebra Realization Report — `bcinr-cmca::allocator::RefusalSet`

**Track:** A — Refusal Algebra Realization
**Writable region:** `crates/bcinr-cmca/src/**`, `crates/bcinr-cmca/tests/**`, this file
**Coordinate:** commit `7e7e7cd5` (branch `recovery/cmca-v26.7.17-c2`, unchanged throughout —
no `git` write command beyond read-only inspection was run by this task), `rustc
1.99.0-nightly (daf2e5e18 2026-07-13)`, `cargo 1.99.0-nightly (59800466c 2026-07-07)`,
`x86_64-apple-darwin` (host default), default features, `dev`/`test` profile.

## Scope and method

`RefusalSet` (`src/allocator.rs`) declares 8 `pub const` bit constants. Its only production
constructor path is `allocate()`'s own `gated_refusals`/`final_refusals` local variables — the
sole place any `RefusalSet` value returned by the crate's authoritative root can come from.
This report determines, for each of the 8 bits, one of the six dispositions named in the task:
REACHABLE, UNREACHABLE_BY_PROOF, OWNED_BY_DIFFERENT_COMPONENT,
RESERVED_WITH_EXPLICIT_NONCLAIM, DEAD_VARIANT_REMOVE, MISSING_IMPLEMENTATION_PATH.

The prior round's finding (quoted in the task) was independently re-derived against the
current tree by direct source reading (`src/allocator.rs` lines 520–585 for the bit
declarations and `primary_reason()`, lines 1445–2018 for `allocate()`'s body, in full) and by
running the existing tests fresh — it was **confirmed accurate**, not assumed. Nothing in
`allocate()`'s body changed between the prior round and this one.

## Per-bit disposition table

| Bit | Disposition | Justification | Test reference |
|---|---|---|---|
| `NO_LEAVES` | **REACHABLE** | Unioned unconditionally on `nl_is_zero` in `allocate()` (allocator.rs:2016), independent of `has_refusal`'s gating — a leafless candidate forest is a structural property of the input, not a control-plane check. | `tests/jtbd_refusal_invariance_regression.rs::no_leaves_only_refusal_leaves_full_state_invariant` (real `allocate()` call, ring-topology `parent` clears every leaf; asserts `refusals() == RefusalSet::NO_LEAVES` exactly and full byte-invariance of `weights`/`last_switch_t`/`prev_mode`) |
| `DIGEST_MISMATCH` | **REACHABLE** | Unioned on `digest_err` (allocator.rs:2003), gated by `has_refusal`. `digest_err` fires when the caller's `digest: [u8; 32]` mismatches the compiled `CERTIFICATE_DIGEST`. | `tests/jtbd_refusal_invariance_regression.rs::digest_mismatch_only_refusal_leaves_full_state_invariant` |
| `DWELL_UNSATISFIED` | **REACHABLE** | Unioned on `dwell_err` (allocator.rs:2004), gated by `has_refusal`. `dwell_err` fires when `tau_d < MODE_DWELL_ROUNDS_MIN`. | `tests/jtbd_refusal_invariance_regression.rs::dwell_unsatisfied_only_refusal_leaves_full_state_invariant` |
| `PROPOSAL_REJECTED` | **REACHABLE** | Unioned on `(!gd_ok)\|lr_err\|beta_err\|eta_err\|q_err\|price_err` (allocator.rs:2006–2008), gated by `has_refusal`. | `tests/jtbd_refusal_invariance_regression.rs::proposal_rejected_only_refusal_leaves_full_state_invariant` |
| `AUTHORITY_MISSING` | **UNREACHABLE_BY_PROOF** | The construction site exists and runs on every call (`.union(RefusalSet::AUTHORITY_MISSING.masked(degrade_to_certified_selection as u32))`, allocator.rs:2009), but the surrounding `gated_refusals` bundle is masked again by `has_refusal = (has_error \| (nl_is_zero != 0)) & !degrade_to_certified_selection` (allocator.rs:1986). `AUTHORITY_MISSING`'s own mask requires `degrade_to_certified_selection == true`; `has_refusal` requires `degrade_to_certified_selection == false`. For any boolean `b`, `b & !b == false` — the conjunction is unsatisfiable by construction, a proof from the two conjuncts' own definitions, not an empirical absence. Independently corroborated by `tests/jtbd_bounded_under_pathological_input.rs`'s module doc comment (a separate, pre-existing Track 4 test), which derives the identical mutual-exclusion argument from an (very slightly stale — it quotes `has_refusal` without the later `nl_is_zero` fold-in, which does not affect this argument) reading of the same gate. | `tests/jtbd_refusal_invariance_regression.rs::authority_missing_is_never_actually_set_verified_by_targeted_run` — a real, targeted `allocate()` call with `proof = None` (satisfies `AUTHORITY_MISSING`'s own mask) plus a real digest mismatch (forces `has_error`), asserting `AUTHORITY_MISSING` is never observed set and the call reports no refusal at all under the current design |
| `ROUND_MISMATCH` | **OWNED_BY_DIFFERENT_COMPONENT** | No code path in `allocate()` constructs this bit (grep-confirmed: appears only in its own `pub const` declaration and in `primary_reason()`'s read-only pattern match). The condition it names — a caller-supplied round identity not matching the round a chain artifact was produced for — is realized, tested, and passing via two other modules' own typed return types, never via `RefusalSet`: `proposal::ProposalRefusal::RoundIdentityMismatch` (`admit_proposal` refuses when `proposal.round_identity != expected_round_identity`) and `certification::CertificationRefusal::RoundIdentityMismatch` (one of the eleven sealed bindings `seal_certificate` independently re-verifies, per `authority-and-c3.md` Invariant 3). | `proposal::tests::refuses_on_round_mismatch`, `certification::tests::refuses_solo_mismatch_round_identity` (both real, both pass — confirmed by `cargo test -p bcinr-cmca --lib` below) |
| `CERTIFICATE_STALE` | **OWNED_BY_DIFFERENT_COMPONENT** | No code path in `allocate()` constructs this bit. "A previously-valid certificate is no longer current" is realized, tested, and passing via `mode_switch::ModeSwitchRefusal::CertificateDigestMismatch` (`apply_mode_switch` refuses when `certificate != expected_certificate` — exactly what a superseded certificate produces against a freshly re-derived expectation) and, for the specific "sealed against a superseded round" sub-case, `certification::CertificationRefusal::RoundIdentityMismatch` (shared ownership with `ROUND_MISMATCH` above — the finer-grained module has one binding check where the coarser `RefusalSet` vocabulary declares two separate bits; this overlap is recorded here rather than papered over). | `mode_switch::tests::rejection_cause_certificate_mismatch_leaves_state_untouched`, `certification::tests::refuses_solo_mismatch_round_identity` (both real, both pass) |
| `CERTIFICATE_MISSING` | **RESERVED_WITH_EXPLICIT_NONCLAIM** | No code path anywhere in the crate constructs this bit — not even a masked-to-zero attempt like `AUTHORITY_MISSING`'s. "No certificate was ever presented" has no representable trigger given the current API surface: `allocate()`'s `digest: [u8; 32]` parameter is mandatory (never `Option<[u8; 32]>`), and `mode_switch::apply_mode_switch`'s `certificate: CertificateReceipt` parameter is likewise mandatory (never `Option`). Both are consequences of the branchless/fixed-shape-input mandate this module opens with ("no input-dependent…branches") and of `numeric-hot-path.md` Invariant 6 (the authoritative root must stay total over a fixed-shape domain) — introducing an `Option` here to distinguish "missing" from "mismatched" would require either a new branch inside the hot path or a caller-side signature change, both outside a documentation-and-test-reconciliation task's scope, and a design decision this track does not have standing to make unilaterally. Not `DEAD_VARIANT_REMOVE`: the underlying domain condition is real and meaningful under `authority-and-c3.md` Invariant 1's four-authority chain (a caller can legitimately never obtain a sealed `CertificateReceipt` at all, e.g. when `seal_certificate` returns `Err` upstream and the caller therefore never calls `apply_mode_switch`), and the bit is already read meaningfully by `RefusalSet::primary_reason()`. Reserved for a future API shape able to distinguish the two cases at a checked boundary. | N/A by definition — no construction path exists to test. Absence corroborated by `tests/jtbd_refusal_invariance_regression.rs::certificate_missing_stale_round_mismatch_have_no_allocate_construction_path` and the representative sweep `no_dead_refusal_bit_appears_across_a_representative_sweep_of_real_allocate_calls` (6 real `allocate()` scenarios, bit checked absent in every one) |

No bit was found to warrant `DEAD_VARIANT_REMOVE` or `MISSING_IMPLEMENTATION_PATH`. Every
declared bit either has a real construction path (in `allocate()` or in an owning C3-chain
module), a proof of unsatisfiability, or an explicit, justified nonclaim — none is
undocumented, none is silently vestigial, and none required fabricating a new `allocate()`
runtime path (which would have violated "no output earns standing merely because an agent
produced it").

## Changes made this round

All changes are documentation-only (doc comments) plus one test-file header/prose
reconciliation; **no runtime logic in `allocate()`, `admit_proposal`, `seal_certificate`, or
`apply_mode_switch` was changed**, and no existing assertion was weakened, relaxed, or deleted:

1. `src/allocator.rs` — added a disposition doc comment to each of the 8 `RefusalSet::*`
   `pub const` declarations (bit, disposition, justification, test reference), so the
   reconciliation is discoverable from the type definition itself, not only from this report.
2. `src/proposal.rs` — added a cross-reference doc comment on
   `ProposalRefusal::RoundIdentityMismatch` naming it as an owning realization of
   `RefusalSet::ROUND_MISMATCH`.
3. `src/certification.rs` — added a cross-reference doc comment on
   `CertificationRefusal::RoundIdentityMismatch` naming it as an owning realization of both
   `RefusalSet::ROUND_MISMATCH` and `RefusalSet::CERTIFICATE_STALE`.
4. `src/mode_switch.rs` — added a cross-reference doc comment on
   `ModeSwitchRefusal::CertificateDigestMismatch` naming it as an owning realization of
   `RefusalSet::CERTIFICATE_STALE`.
5. `tests/jtbd_refusal_invariance_regression.rs` — rewrote the module-level doc comment and
   the two per-section comments to record the full 6-way disposition vocabulary (rather than
   the prior binary "reachable/unreachable" framing) and to cite the owning-module test names
   for `ROUND_MISMATCH`/`CERTIFICATE_STALE`. No test function, assertion, or fixture was
   changed.

All 4 REACHABLE bits already had a dedicated same-object test (constructed via the real
`allocate()`) proving byte-invariance in `tests/jtbd_refusal_invariance_regression.rs`; none
needed to be added. `AUTHORITY_MISSING`'s targeted-unreachability test already existed and
needed no change. The two `OWNED_BY_DIFFERENT_COMPONENT` bits' owning-module tests
(`proposal.rs`, `certification.rs`, `mode_switch.rs`) already existed, independent of this
round's work, and pass.

## Adjacent finding, out of this track's explicit objective (reported, not fixed)

`RefusalSet::primary_reason()` implements a documented priority order over co-occurring bits
(required by `authority-and-c3.md` Invariant 2: "a separate, named, separately tested priority
order layered on top of the full set"). Searching the crate (`grep -rn primary_reason`) found
exactly one indirect exercise of it (`tests/case_studies.rs:313`, a single-cause case) and no
test that constructs a `RefusalSet` with two or more bits simultaneously true and asserts
`primary_reason()`'s tie-break choice against that specific multi-true input. This is a real
gap against Invariant 2's own required-evidence clause, but it is about the *priority
projection's* test coverage, not about the *reachability* of the 8 bits this track's objective
targets — flagged honestly per the governing "report the finding, don't work around it" rule,
left unfixed as out of this track's assigned scope (reconciling `RefusalSet`'s reachability
algebra, not auditing `primary_reason()`'s tie-break testing).

## Precision note on "unreachable"

Every disposition above concerning non-construction by `allocate()` is scoped precisely to
*`allocate()`'s own construction path* — `RefusalSet`'s bit constants and its `union`/`masked`
methods are all `pub`, so any caller (inside or outside the crate) can construct
`RefusalSet::CERTIFICATE_MISSING` or any other bit directly and call `.primary_reason()` on it;
this is by design (the test suite itself does exactly this for assertions). "Unreachable" and
"no construction path" in this report mean "never produced by a real call to the authoritative
root `allocate()`," not "impossible to instantiate via the public API at all." This distinction
is stated explicitly to avoid overclaiming absolute unconstructibility.

## Real test evidence, this round

**`cargo test -p bcinr-cmca --test jtbd_refusal_invariance_regression`** — exit 0:

```
running 8 tests
test certificate_missing_stale_round_mismatch_have_no_allocate_construction_path ... ok
test digest_mismatch_only_refusal_leaves_full_state_invariant ... ok
test no_leaves_only_refusal_leaves_full_state_invariant ... ok
test dwell_unsatisfied_only_refusal_leaves_full_state_invariant ... ok
test authority_missing_is_never_actually_set_verified_by_targeted_run ... ok
test baseline_triggers_no_refusal_at_all ... ok
test proposal_rejected_only_refusal_leaves_full_state_invariant ... ok
test no_dead_refusal_bit_appears_across_a_representative_sweep_of_real_allocate_calls ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**`cargo test -p bcinr-cmca --lib`** (owning-module same-object tests cited in the table
above, confirmed real and passing) — exit 0, 63 passed, 0 failed, including:

```
test certification::tests::refuses_solo_mismatch_round_identity ... ok
test proposal::tests::refuses_on_round_mismatch ... ok
test mode_switch::tests::rejection_cause_certificate_mismatch_leaves_state_untouched ... ok
test mode_switch::tests::rejection_cause_stale_admitted_state_leaves_state_untouched ... ok
```

**`cargo test -p bcinr-cmca`** (full default-feature suite) — exit 0:

```
test result: ok. 63 passed; 0 failed  (lib unit tests)
test result: ok. 0 passed; 0 failed   (alloc_gate.rs — feature-gated, not part of default run)
test result: ok. 6 passed; 0 failed   (calibration.rs)
test result: ok. 7 passed; 0 failed   (case_studies.rs)
test result: ok. 1 passed; 0 failed   (compile_fail_tests.rs — 44 nested trybuild cases)
test result: ok. 3 passed; 0 failed   (consumer_correspondence.rs)
test result: ok. 1 passed; 0 failed   (differential.rs)
test result: ok. 5 passed; 0 failed   (hostile_mutants.rs)
test result: ok. 9 passed; 0 failed   (jtbd_artifact_lifecycle.rs)
test result: ok. 2 passed; 0 failed   (jtbd_auditable_adaptive_policy.rs)
test result: ok. 8 passed; 0 failed   (jtbd_boundary_adversarial_inputs.rs)
test result: ok. 1 passed; 0 failed   (jtbd_bounded_under_pathological_input.rs)
test result: ok. 1 passed; 0 failed   (jtbd_conservation_regression.rs)
test result: ok. 3 passed; 0 failed   (jtbd_multi_agent_resource_governance.rs)
test result: ok. 8 passed; 0 failed   (jtbd_refusal_invariance_regression.rs)
test result: ok. 2 passed; 0 failed   (jtbd_safety_certified_adaptive_control.rs)
test result: ok. 10 passed; 0 failed  (jtbd_semantic_mechanical_compilation.rs)
test result: ok. 2 passed; 0 failed   (jtbd_sequential_state_evolution.rs)
test result: ok. 0 passed; 0 failed   (reference.rs)
test result: ok. 15 passed; 0 failed  (doc-tests)
```

**Total: 147 tests passed, 0 failed, 0 ignored** — identical count and identical pass status to
the pre-edit baseline captured at the start of this task (also 147/0/0, re-run and diffed
before making any change), confirming this round's documentation-only edits introduced no
regression. The 4 pre-existing `unused import` warnings (`CanonicalMask`/`SignedFixed`/
`bcinr_cmca::generated::generalization as gen`) are unchanged in count before and after this
round's edits (`grep -c "unused import"` on both full-suite logs: 4 and 4) — not introduced by
this task.

`cargo build -p bcinr-cmca` — exit 0, clean compile after every doc-comment edit.

## Governing-law compliance

- **Repository reality**: every disposition above was reconciled against the current tree by
  direct file reads (`allocator.rs`, `proposal.rs`, `certification.rs`, `mode_switch.rs`,
  `stability.rs`, `observatory.rs`, `jump.rs` — the latter three read to confirm no other
  module owns any of the 3 unconstructed bits under a different name) and grep confirmation
  that `RefusalSet` is imported and used only inside `allocator.rs`.
- **Same-object falsification**: every REACHABLE/OWNED_BY_DIFFERENT_COMPONENT test cited
  above calls the real, unmodified production function (`allocate()`, `admit_proposal()`,
  `seal_certificate()`, or `apply_mode_switch()`) — no fixture or synthetic `RefusalSet` value
  stands in for a production call.
- **No self-certification**: this report was produced by the same track that made the doc
  edits; it does not itself constitute a second, independent verification pass — the "Real
  test evidence" section above is the reproducible artifact a separate verifier would re-run.
- **Zero unreceipted actuation**: no runtime state, packaging, or standing transition was
  touched; every change is a `.rs` doc comment or test-file prose edit, verified by a real
  `cargo test`/`cargo build` run captured above.
- **Fix forward only / no destructive git**: branch unchanged (`recovery/cmca-v26.7.17-c2`
  throughout, confirmed via `git branch --show-current`), no commits made by this task, no
  `git reset`/`checkout -- `/`clean` run.

## Verdict

All 8 declared `RefusalSet` bits now have one of: a reachable, same-object-tested construction
path in `allocate()` (4 bits); a mathematical proof of unreachability plus a targeted
confirming test (1 bit); a traced, tested, real owning realization in a named different C3-chain
module (2 bits); or an explicit, justified nonclaim with no fabricated trigger (1 bit). No bit
is undocumented, silently dead, or fabricated. The full default-feature test suite (147 tests)
passes identically before and after this round's changes.

CMCA_REFUSAL_ALGEBRA_ALIVE
