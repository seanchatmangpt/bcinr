# `bcinr_cmca::sa2a` — SA2A Third-Consumer Boundary

`bcinr_cmca::sa2a` is the third-consumer boundary of the CMCA crate. It consumes
powerless `PreparedEffect`, resource and receipt identities only: it never mints
authority and never performs effects. Every type below is arithmetic or
validation state; nothing here carries a SELECT/DO surface.

- Contract string: `CONTRACT` = `"sa2a/replan-envelope/v1"`
- Source: `crates/bcinr-cmca/src/sa2a/` (`mod.rs` re-exports the public API; the
  crate exposes the module unconditionally, no feature gate)

## Admission and envelopes

| Item | Kind | Meaning |
|---|---|---|
| `Envelope<'a>` | struct | `{ contract, subject, effect_id, replay_id, authority }`. `Envelope::powerless(subject, effect_id, replay_id)` is the only intended constructor; it fixes `authority: "none"`. |
| `admit(&Envelope) -> Result<(), Refusal>` | fn | Refuses unless the contract matches `CONTRACT`, `authority == "none"`, and `subject`, `effect_id`, `replay_id` are all non-empty. |
| `Refusal` | enum | `Contract`, `Authority`, `EmptySubject`, `EmptyEffect`, `EmptyReplay` — one per failed check, in check order. |
| `RecoveryDecision` | enum | `Reconcile`, `Replan`, `Terminal`. |

`admit` refuses any envelope that claims authority (`Refusal::Authority`); a
powerless identity is the only admissible shape.

## Resource budget ledger

| Item | Kind | Meaning |
|---|---|---|
| `ResourceEnvelope` | struct | `{ cpu, memory, io }` (u64 each). `ResourceEnvelope::ZERO`; `admits(&Allocation)`; `child(&Allocation) -> Option<Self>` (residual after a bounded allocation — arithmetic state, never actuation authority); `granted(&Allocation)`. |
| `Allocation` | struct | `{ cpu, memory, io }` — a bounded resource request. |
| `BudgetLedger` | struct | `{ capacity, remaining }`. `new(capacity)` starts `remaining == capacity`; `propose(Allocation) -> Option<BudgetReceipt>`; `advance(BudgetReceipt) -> Option<Self>`. |
| `BudgetReceipt` | struct | `{ before, granted, after }`. `conserves() -> bool`; `child_ledger() -> BudgetLedger`. |

### Conservation invariant

A `BudgetReceipt` conserves iff `after + granted == before` on every axis under
checked (overflow-refusing) arithmetic. `BudgetLedger::advance` accepts a
receipt only when its `before` equals the ledger's current `remaining` **and**
it conserves, so total budget is never created or destroyed as allocations move
through the ledger. A recursive allocator receives exactly its parent's grant
via `BudgetReceipt::child_ledger` — never the parent's residual. The ledger
checks and accounts conservation only: it does not mint an actuation
certificate and has no DO surface.

## Symbolic envelope

| Item | Kind | Meaning |
|---|---|---|
| `SymbolDomain` | struct | One source-traceable symbolic integer domain: values form the progression `lower, lower + alignment, ... <= upper`. `contains(u64)`, `cardinality()`, `contains_domain(&Self)`. Zero alignment or `lower > upper` is invalid (cardinality 0). |
| `AffineResourceExpr<const V: usize>` | struct | Non-negative affine expression `base + Σ coeffs[i] * symbol[i]`; `evaluate(&[u64; V]) -> Option<u64>` refuses on overflow. |
| `SymbolicFamily<const V: usize>` | struct | One discrete candidate family with symbolic value choices: `{ subject_id, family_id, reachability_mask, domains: [SymbolDomain; V], cpu/memory/io: AffineResourceExpr<V> }`. `concrete_space_saturating()`, `concretize(&SymbolicWitness) -> Result<Allocation, SymbolicRefusal>`, `admit_in(&ResourceEnvelope, &SymbolicWitness)`, `subsumes_without_option_loss(&Self)`. |
| `SymbolicWitness<const V: usize>` | struct | Solver-proposed concrete values `{ family_id, values: [u64; V] }`. Possession is not authority: it must pass `admit_in`. |
| `SymbolicRefusal` | enum | `FamilyIdMismatch`, `InvalidDomain`, `ValueOutOfDomain`, `ArithmeticOverflow`, `EnvelopeExceeded`, `InvalidLifetime`. |
| `TimedAllocation` | struct | Logical resource use over the half-open lifetime `[start, end)`. |
| `peak_reusable_usage(&[TimedAllocation; M]) -> Result<Allocation, SymbolicRefusal>` | fn | Physical peak when non-overlapping logical lifetimes reuse capacity; probes start points only (the active set can only increase at a start), bounded O(M²), stack-only; refuses inverted lifetimes. |
| `PipelineKnobs` | struct | Load/body/store pipeline-boundary choices `{ load_body_open, body_store_open }`. |
| `pipeline_latency(load, body, store, PipelineKnobs) -> Result<u64, SymbolicRefusal>` | fn | Composes the load/body/store costs using the HEG pipeline equations (arXiv:2609.29219); open boundaries overlap via `max`. Models overlap only; selects no candidate. |

### Invariants

- **Symbolic until solved, exact at concretization.** A `SymbolicFamily` keeps
  high-cardinality value choices symbolic; a solver (CP-SAT, SMT, a planner, or
  a human) proposes a `SymbolicWitness`, and `concretize`/`admit_in` recheck
  everything from scratch: family id, per-slot domain membership, checked
  affine evaluation, then envelope admission. The representation is
  solver-agnostic and no trust is placed in the proposer.
- **CONSTRUCT-only.** A symbolic witness is analytical: the module can validate
  it against a `ResourceEnvelope` but cannot SELECT a family or DO. (CMCA
  deliberately does not import Loom's final minimum-latency selection
  authority.)
- **Option-preserving pruning.** `subsumes_without_option_loss` permits one
  family to replace another only on exact set containment: same `subject_id`,
  identical resource expressions, `reachability_mask` preserving every
  reachable edge, and every domain a subset. No heuristic ranking.

## Other submodules

`sa2a` also hosts `forecast_benchmark`, `forecast_standing`,
`predictive_envelope`, `predictive_measure`, `predictive_model`, `reservation`
and `shadow_optimizer`, re-exported from `bcinr_cmca::sa2a` (see the crate docs
for those surfaces).
