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
