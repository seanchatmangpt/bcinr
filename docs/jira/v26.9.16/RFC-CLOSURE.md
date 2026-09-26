# bcinr v26.9.16 — RFC Closure Contract

Status: DRAFT IMPLEMENTATION PR.

## Canonical Jira tickets

- A2A-2605 — CMCA bounded resource-allocation control plane
- A2A-2606 — recursive MFW / DME closure
- A2A-2612 — machine-experience compile-back

## RFC ownership

This repo owns the bounded formal planning/allocation substrate used by the DME architecture:

- admitted planning tasks and capability profiles;
- PDDL/POWL/Prolog admission and execution receipts;
- MFW frontier, q-lens, rails, consequence horizons and bounded epochs;
- cost/capability vectors needed by CMCA;
- LLM-first-mile candidate admission without authority;
- reusable deterministic planning machinery produced from admitted discoveries.

## Required closure

1. Expose a stable allocator-facing interface over existing `FrontierMeasure`, `MassVector`, q-lens, `CostVector`, fair-rail and consequence-horizon machinery.
2. Represent the DME work classes `KNOWN`, `UNKNOWN_LOCAL`, `UNKNOWN_DEFERRED`, `UNKNOWN_FRONTIER`, `REFUSED`, `BLOCKED`, `UNSUPPORTED` without granting execution authority.
3. Guarantee hard epoch/budget/fan-out/depth ceilings; exhaustion returns a typed witness instead of silently expanding search.
4. Provide a machine-experience compilation contract that can turn a qualified admitted solution into a reusable capability/planning artifact with exact content identity.
5. Preserve the existing law: LLM proposes; substrate admits.

## Chicago falsifiers

- requesting additional search mass automatically increases the budget;
- equivalent KNOWN work enters an exploratory LLM/search rail when an admitted deterministic route exists;
- bound exhaustion is represented as success;
- candidate PDDL/domain text bypasses admission;
- compiled experience changes semantic identity without changing its digest;
- a planning artifact acquires DO authority.

## Definition of done

Repository-native exact-head verification proves bounded allocation, bounded exhaustion, deterministic reuse, admitted compile-back, and receipts for each transition. The resulting interfaces are consumed by the ggen and ash_a2a v26.9.16 branches without duplicate planner/allocation implementations.

## v26.9.26 hardening (on top of ff0b7647)

Defects found by adversarial probes against the PR head, each now a permanent guard:

| defect at ff0b7647 | observed | guard |
|---|---|---|
| route decision digest depended on candidate order | reversed candidate list: `digest_equal=false` | canonical candidate order + dedup; `every_permutation_of_routes_yields_the_identical_sealed_decision`, proptest (512 cases) |
| decision digest omitted consequence/deadline/evidence/frontier/routes | stale decision `verifies=true` against a changed request | canonical request bound into the digest (domain `bcinr-cmca/dme-route-decision/v2`); `decision_is_bound_to_every_semantic_request_field` |
| blank `request_id` admitted | `accepted=true` | `DmeRouteRefusal::InvalidRequestId` |
| epoch identity depended on residual order | `same_id=false` | residuals stored/hashed as a sorted set |
| feedback could substitute fresh obligations | minted residual `accepted=true` | set descent; `EpochRefusal::ResidualNotInParent` |
| fresh meter re-minted depth for a deeper epoch | grandchild `depth=1` under parent depth 1 | `EpochRefusal::StaleDescentMeter` |
| `experience.rs` not declared as a module | contract never compiled or tested | `pub mod experience` + re-exports + `tests/experience_chicago.rs` |
| execution receipt could double as admission/verification receipt | not checked | `ExperienceRefusal::NonIndependentReceipts`; domain-separated identities |
| `Cargo.lock` missing `blake3` for `bcinr-cmca` | lock diff | lock regenerated offline |
| crate docs and the `dummy_branchless` doctest deleted by the PR | doctest count dropped | docs restored; PR additions kept |

Anti-vacuity: 10/10 source mutants of the new guards are killed by the new tests.
Benchmarks: `crates/bcinr-cmca/benches/dme_route.rs`, `crates/bcinr-mfw-ir/benches/dme_epoch.rs`;
numbers and regression bounds in `bench/dme-bench-receipt.json`; bounds enforced by
`tests/dme_route_perf_regression.rs` and `tests/dme_epoch_perf_regression.rs`.

## v26.9.26 hardening, round 2 (on top of 2faac611)

Spec-audit and court findings against 2faac611, each closed with code and a Chicago test:

| finding | guard |
|---|---|
| closure item 1: no allocator-facing interface over the existing machinery | `bcinr_pddl::dme_allocator` (feature `mfw-planner`): `CostVector` rank + `MassVector`/`FrontierBoxes`/`q_lens` weight -> `RouteCandidate`; `FairRailScheduler` tick schedule; `ConsequenceHorizon` id bound into the decision digest; composed into `bcinr_cmca::classify_dme_work` (no second selector); `tests/dme_allocator_chicago.rs` |
| closure item 2: `UNKNOWN_DEFERRED`, `BLOCKED`, `UNSUPPORTED` missing | `RouteClass::UnknownDeferred`; `DmeWorkClass` (7 classes) + `classify_dme_work`; `all_seven_dme_work_classes_are_reachable_and_authority_free` |
| closure item 3: exhaustion carried no witness | `DmeRouteRefusal::{Known,Unknown}Without*(DmeExhaustionWitness)`; `EpochRefusal::DescentBoundHit(BoundHit)`; `EpochRefusal::DescentBudgetAboveCeiling(BoundHit)`; allocator `FanOutCeiling`/`BudgetCeiling`/`ScheduleCeiling(BoundHit)` |
| falsifier "requesting additional search mass automatically increases the budget" had no test | `requesting_more_search_mass_never_increases_the_budget` (cmca), `requesting_more_search_mass_never_increases_the_admitted_budget` (allocator): over-budget demand is `BLOCKED` with the exact shortfall |
| falsifier "candidate PDDL/domain text bypasses admission" had no test | `WorkAdmission::Candidate` is routed with standing `Candidate` and refused; `AdmittedWork` is constructible only from `AdmittedDomain` / `admit_domain_work` (the real `admit_candidate_domain` gate); `candidate_domain_text_never_reaches_a_route` |
| forged semantic subject kept its capability id (probe P1) | `PromotionCandidate` fields private (compile_fail doctest); `qualify_candidate` recomputes the candidate id (`CandidateIdentityMismatch`); capability id binds `semantic_subject` (domain `capability/v2`) and `QualifiedCapability` carries it |
| hand-built candidate from failed execution qualified (P2, A3) | sealed constructor: `candidate_from_execution` is the only path |
| `Digest::ZERO` accepted as a receipt (A4) | `ExperienceRefusal::NullReceipt` |
| forged parent residuals / duplicate parent residuals descended (P3, A1, A2) | `DmeEpoch` fields private (compile_fail doctest); `advance_epoch` recomputes the parent identity from canonical contents (`ParentIdentityMismatch`) |
| caller-chosen meter budget (P4) | budget bound into epoch identity (domain `dme-epoch/v2`), hard ceiling `MAX_DME_EPOCH_DESCENT_BUDGET = 64`, wider meter refused (`ForeignDescentBudget`) |
| selected class also listed as refused (B2) | a class is refused only when no candidate of that class is eligible |
| clippy `type_complexity` in `dme_route.rs` lib | `CanonicalKey` / `SelectionKey` aliases; lib clippy `-D warnings` clean |
| epoch perf guard killed the quadratic mutant 3-4 of 6 | guard moved to n=512/n=4096 with ratio < 20 and absolute < 60 ms; mutant fails both bounds every run (see bench receipt) |

Anti-vacuity: 14/14 source mutants of the round-2 guards are killed by the new tests.
