# 04 — Ledger as State Machine

**Status:** DESIGN document for a future post-v26.7.17 milestone ("Constitutional Compiler
v0"). No code in this document is wired to dispatch anything. Nothing here changes how
V26_7_17_RELEASE_LEDGER.md is currently being advanced.

## Scope statement (read this first)

This document formalizes the release ledger — currently a hand-authored Markdown file,
`docs/cmca-rdf/V26_7_17_RELEASE_LEDGER.md` — as an executable state machine: a generic gate
tuple plus three transition laws that a future scheduler could evaluate mechanically. It is
a specification only.

Two facts anchor that scope boundary:

1. The gates G0-G9 in the actual ledger are, as of this writing, being advanced by a
   separate, currently-running background workflow via a hand-written script — not by any
   state machine, and not by anything in this document. This document does not touch, read
   the live state of, or influence that workflow.
2. No agent dispatch, no automated `RepairIntent`/`VerifyIntent` invocation, and no
   `RecomputeStanding` mechanism exists anywhere in the codebase today. The transition laws
   below describe what a future implementation would need to satisfy — they are not a
   report of an existing scheduler, and this document does not create one.

## 1. The generic gate tuple

A gate `g` in the abstract ledger model is a 9-tuple:

```
g = (id, standing, owner, verifier, dependencies, falsifier,
     repair_capability, verification_capability, evidence_requirements)
```

| Field | Type (informal) | Meaning | Ledger precedent |
|---|---|---|---|
| `id` | string | Stable gate identifier | `G0`..`G9` headers in the ledger |
| `standing` | enum | Current status: `REPORTED \| CONFIRMED \| ALIVE \| BLOCKED \| PARTIAL_ALIVE \| ...` | the `**Standing:**` line under each gate |
| `owner` | agent-ref | Agent responsible for driving the gate toward closure | the `**Owner:**` line |
| `verifier` | agent-ref | Agent responsible for independently reproducing the owner's claims | the `**Verifier:**` line |
| `dependencies` | set of gate-ids | Gates that must reach a qualifying standing before this gate can close | implicit in the ledger prose (e.g. G9 depends on G0-G8) |
| `falsifier` | predicate | The concrete condition that would prove the gate's claimed standing false | the `**Falsifier:**` line |
| `repair_capability` | capability-ref | What the owner invokes to move a `BLOCKED` gate toward a repaired state | the `**Blocker:**` line's implied next action |
| `verification_capability` | capability-ref | What the verifier invokes to reproduce a claim against cited evidence | the `**Required evidence/commands:**` line |
| `evidence_requirements` | list of artifacts | The concrete commands/outputs a reproduction must produce to justify a standing change | same `**Required evidence/commands:**` line, itemized |

The ledger today expresses all nine fields per gate, but as unstructured prose under
Markdown bullet headers rather than as typed tuple fields. Section 3 below is a worked
mapping of the *actual* G0-G9 entries onto this tuple shape, to show the fit is not
theoretical.

## 2. The three transition laws

These are the state-transition rules an executable ledger would enforce. Each is written as
an implication — a triggering condition on the left, a dispatch or recomputation obligation
on the right. None of these are implemented; none of the referenced `Dispatch`,
`RepairIntent`, `VerifyIntent`, or `RecomputeStanding` names exist as code anywhere in this
repository as of this writing.

### Law 1 — Blocked standing obligates a repair dispatch

```
Standing(g) = BLOCKED  =>  Dispatch(Owner(g), RepairIntent(g))
```

Reading: whenever a gate's standing is `BLOCKED` (i.e. its own `**Blocker:**` field
identifies an open decision or defect that must close before the gate can advance), the
state machine's only lawful reaction is to dispatch the gate's `owner` agent with a
`RepairIntent` carrying the gate's `falsifier`, `dependencies`, and `blocker` text as
payload. No other agent may be dispatched for repair — ownership is exclusive per the
ledger's stated ownership matrix.

### Law 2 — Artifact change obligates a verification dispatch

```
ArtifactChanged(g)  =>  Dispatch(Verifier(g), VerifyIntent(g))
```

Reading: whenever any artifact cited in a gate's `evidence_requirements` (a file, a command
output, a digest) changes on disk relative to what the ledger's current entry describes, the
state machine dispatches the gate's `verifier` with a `VerifyIntent` — an instruction to
re-run the cited reproduction command against the new artifact state and report a match or
mismatch. This is the mechanism that would eventually retire the ledger's own standing
discipline note ("no entry here may be read as CONFIRMED ... until reproduction happens").

### Law 3 — A valid receipt obligates a standing recomputation

```
ReceiptValid(g)  =>  RecomputeStanding(g)
```

Reading: whenever a verification or repair dispatch returns a cryptographically or
mechanically checkable receipt (e.g. a BLAKE3 digest match, a passing reproduction command,
a byte-identical diff) that the state machine can validate independently of the dispatched
agent's own self-report, the state machine recomputes `standing(g)` from that receipt plus
the current `dependencies` closure — it does not accept the dispatched agent's claimed
standing directly. This is the formal analogue of the ledger's existing rule that
`cmca-verifier` (or, for G6, `cmca-release-integrator`) — never the owner alone — is the
party whose reproduction actually changes a gate's tag from `REPORTED` toward `CONFIRMED`.

### Why three laws, not one loop

The three laws are independent triggers, not sequential steps of one loop: `BLOCKED` can
recur after a `RecomputeStanding` step lowers a gate's standing again (a regression); an
`ArtifactChanged` event can fire while a gate is not `BLOCKED` at all (proactive
re-verification); and `ReceiptValid` can arrive out of band, e.g. from a scheduled nightly
reproduction sweep rather than from a dispatch this state machine itself issued. Treating
them as three separately-armed rules rather than one linear pipeline is what makes the
design tolerant of a gate reopening (an artifact regressing after a prior CONFIRMED tag) —
a case the current hand-written script has no formal obligation to handle beyond
"the ledger gets updated to reflect it," and the current ledger already does state this is
possible ("re-tagged accordingly").

## 3. Worked example — the actual G0-G9 gates mapped onto the tuple shape

The following table maps the nine gates as they stand *today* in
`docs/cmca-rdf/V26_7_17_RELEASE_LEDGER.md` onto the tuple's `standing`, `owner`, and
`verifier` fields, plus what a `RepairIntent`/`VerifyIntent` dispatch would concretely carry
for each gate under Law 1 / Law 2 above. This is illustrative only — issuing any of these
intents is not part of this task, and none of them have been issued.

| Gate | Standing (per ledger, 2026-07-17) | Owner | Verifier | What `RepairIntent(g)` would concretely mean | What `VerifyIntent(g)` would concretely mean |
|---|---|---|---|---|---|
| G0 — Release Identity | REPORTED; blocked on version-bump/toolchain-pin decision | cmca-release-integrator | cmca-verifier | Decide whether `bcinr-cmca`/`bcinr-logic` share a synchronized `26.7.17` bump and resolve the `rust-toolchain.toml` nightly-vs-`rust-version=1.70` tension, then edit the `Cargo.toml`/`rust-toolchain.toml` fields accordingly | Re-run `grep '^version' crates/bcinr-cmca/Cargo.toml crates/bcinr-logic/Cargo.toml` and `cargo publish --dry-run -p bcinr-cmca` against the post-repair tree and confirm the version strings and dry-run both match the decided target |
| G1 — Workspace/Packaging Hazards | REPORTED; blocked on whether the `wasm4pm-compat` absolute-path patch blocks dry-run on a foreign machine | cmca-release-integrator | cmca-verifier | Remove or condition the `[patch.crates-io]` absolute-path override and clear the dirty-tree scratch files (`allocator.rs.orig`, `err_list.txt`, `errors.json`, `bcinr-cmca.s`, `cmca_dump.txt`, `objdump.txt`) | Re-run `cargo publish --dry-run -p bcinr-cmca` on a checkout without `/Users/sac/wasm4pm-compat` present, plus `git status --porcelain` and `git clean -ndx`, and confirm no scratch artifacts remain and the dry-run behavior matches whatever the repair decided |
| G2 — Numeric Law | REPORTED; blocked on the missing `NumericFaultSet`/`RefusalSet` type decision and the floor's conservation proof | cmca-numeric | cmca-verifier | Decide and implement (or explicitly reject) an encapsulating fault-channel type for `NonNegativeFixed`/`SignedFixed`/`CanonicalMask`/`allocate`, fix the `from_bits(const_select_u32(...))` fault-dropping sites, and reconcile the two `const_eq_u32` signatures | Re-run the targeted `cargo test -p bcinr-cmca` cases per sub-obligation (fault survival under selection, first-wins vs union semantics, zero-denominator/zero-`nl` fault recording, byte-level rollback equality) against the repaired code and confirm each assertion now holds |
| G3 — Authority: Certification/Sealing | REPORTED; blocked on whether existing trybuild/compile_fail tests cover the 11-category authority model | cmca-authority | cmca-verifier | Implement `seal_certificate` as a distinct step from `evaluate_calibration`, and/or extend `admit_adaptive_update` to bind all 11 required categories (not ~0/11) | Attempt to construct a `CertificateReceipt` from outside `observatory.rs`'s module boundary (expect compile failure) and enumerate which of the 11 categories `admit_adaptive_update` now checks, confirming full coverage |
| G4 — Authority: Mode-Switch Lifecycle | REPORTED; blocked on implement-vs-accept-inline-design policy decision | cmca-authority | cmca-verifier | Either implement the proposal/shadow/jump/stability/dwell/certified-switch typed lifecycle (new files, new types per the ledger's sub-obligations 1-2), or formally accept the inline-masked-select design as v26.7.17 scope | Re-run the `find`/`grep` commands for the named files and types and confirm they now exist (implement path) or confirm the ledger has been updated to record the accepted-inline decision (accept path) |
| G5 — Semantics: RDF/Ontology Generator | REPORTED; superseded in scope by the C4_mfw_admission / C4_projection / C4_bcinr_consumption split recorded in the ledger's Architecture Correction section | cmca-semantics | cmca-verifier | Execute migration steps 3-10 of the accepted 10-step sequence (port generator into mfw, generate byte-equivalent payloads, resolve the 11 semantic defects in the mfw-hosted generator, remove RDF parsing from bcinr) | Run mfw's admit-graph -> validate -> receipt pipeline over the CMCA ontology inputs and confirm a reproducible BLAKE3 digest; separately confirm `bcinr-cmca` contains no RDF/Turtle parsing post-migration |
| G6 — Verifier: Object-Code/Branchlessness Closure | REPORTED; blocked because no object-code audit tooling exists to run | cmca-verifier | cmca-release-integrator (G6 is the self-verification exception) | Build or manually execute the `object-code-audit` and `mutant-kill-protocol` skills' procedures against a release build; wire `test-mutants` to all 11 declared mutant features and add it to the `ci` task dependency list | Independently re-run the same disassembly/mutant-kill procedure `cmca-release-integrator` used and confirm no conditional branch is found on a claimed-branchless input, and that all 11 mutants are now wired and killed |
| G7 — Documentation/Standing Consistency | REPORTED; blocked on a supersede-vs-deprecate decision for `CURRENT_STATUS.md` | cmca-release-integrator | cmca-verifier | Update `CURRENT_STATUS.md` to cover C1/C2/C3/C6 (not only C4), or mark it deprecated in favor of the v26.7.17 ledger | Re-read `CURRENT_STATUS.md`, `BASELINE.md`, `AUDIT_REPORT.md`, `ARCHITECTURE.md`, `AGENT_DISPOSITION.md` and confirm no remaining standing claim contradicts this ledger's G2-G6 entries |
| G8 — Constitutional/Config Surface | REPORTED; ledger states no blocker beyond standard reproduction | cmca-release-integrator | cmca-verifier | None currently identified — this is the one gate whose repair is expected to be a no-op once reproduced | Re-run the `ls`/`find` commands for `.claude/rules/`, `.claude/agents/*.md`, `.claude/skills/*/SKILL.md` on a clean checkout and confirm the reported presence/absence facts still hold |
| G9 — Standing/Evidence Rollup | REPORTED; blocked until G0-G8 are independently reproduced and re-tagged | cmca-release-integrator | cmca-verifier | Aggregate the re-tagged standings of G0-G8 once each has a `ReceiptValid` recomputation (Law 3) and run the `evidence-report` skill's checklist against the resulting ledger | Confirm the literal token `CMCA_RDF_PARTIAL_ALIVE` appears where and only where the rollup asserts it does, and that the mandated final-standing sentence from `ORIGINAL_REQUEST.md` is reproduced verbatim |

## 4. What this document is not

- Not a scheduler. No queue, dispatcher, or agent-invocation code exists corresponding to
  `Dispatch`, `RepairIntent`, `VerifyIntent`, or `RecomputeStanding` in this repository.
- Not a change to the live ledger. `docs/cmca-rdf/V26_7_17_RELEASE_LEDGER.md` is unmodified
  by this document and continues to be advanced by the separate, currently-running
  background workflow via its own hand-written script.
- Not a claim that G0-G9's REPORTED standings have changed. The standing column above is a
  snapshot transcription of the ledger as read on 2026-07-17; it carries no independent
  verification of its own.

## See also

- `docs/cmca-rdf/V26_7_17_RELEASE_LEDGER.md` — the live, mutable ledger this document
  formalizes; read-only reference for this file, not edited by it.
- `AGENTS.md` — constitution defining the REPORTED/CONFIRMED/ALIVE/PARTIAL_ALIVE vocabulary
  the `standing` field's enum draws from.
- `docs/cmca-rdf/CMCA_ARTIFACT_CONTRACT.md` — the artifact-contract shape referenced by the
  `evidence_requirements` field's ledger precedent.
