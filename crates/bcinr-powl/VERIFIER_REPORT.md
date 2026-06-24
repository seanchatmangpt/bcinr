# VERIFIER_REPORT.md — bcinr-powl 80% Hostile-Audit Admissibility

## Summary

bcinr-powl has been evaluated against the 5-track hostile-audit checklist. At the 80% threshold, the artifact demonstrates: clean compilation on nightly Rust, 160/163 library tests passing in no_std and std modes respectively, a working OCEL 2.0 export path, conformance validation via `validate_against_tape`, enforced loop bounds, XOR-inside-loop rejection at compile time, receipt topo-order verification, ring overflow instrumentation, two-phase Kahn cycle detection, reachability checking, and a branchless hot-path benchmark. Excluded from admission at this threshold: SOTA crown (requires external baselines not yet run), enterprise-grade replacement claims (out of scope), and external substrate verification (targeted for 90%). The 80% threshold means the artifact is internally self-consistent, reproducible, and carries sufficient proof to enter hostile review — not that it has been independently corroborated.

## Admitted State

bcinr-powl is hereby admitted as a hostile-audit admissible proof-carrying POWL research artifact.

## What This Is Not

- Not a full Camunda/Temporal replacement
- Not independently SOTA-crowned
- Not peer-reviewed

## Status Table

| Claim | Status | Evidence |
|---|---|---|
| Builds cleanly | ADMITTED | `cargo check -p bcinr-powl` → Finished (0 errors, 0 warnings) |
| Tests pass (no_std) | ADMITTED | 160 passed; 0 failed |
| Tests pass (std) | ADMITTED | 163 passed; 0 failed |
| OCEL export path | ADMITTED | `to_ocel_2_0` + `to_ocel_json` implemented |
| Conformance validation | ADMITTED | `validate_against_tape` |
| Loop bounds enforced | ADMITTED | `iter_under_limit` + `apply_loop_redo` gate |
| XOR inside loop rejected | ADMITTED | `CompileError::XorInsideLoop` |
| Receipt topo-order | ADMITTED | `verify_topo_order` |
| Ring overflow instrumented | ADMITTED | `PetriTickResult.event_overflow_count` |
| Cycle detection (all cycles) | ADMITTED | two-phase Kahn |
| Reachability check | ADMITTED | `check_all_ops_reachable` |
| Branchless hot path | MEASURED | `bench_branchless_gate` |
| SOTA crown | NOT ADMITTED | requires external baselines |
| Enterprise replacement | NOT ADMITTED | out of scope at 80% |

## Build Metadata

| Field | Value |
|---|---|
| `rustc --version` | rustc 1.98.0-nightly (3daae5e42 2026-06-14) |
| `cargo --version` | cargo 1.98.0-nightly (fe63976b2 2026-06-11) |
| Report date | 2026-06-24 |
| Branch | main |
| HEAD | 3bdffe45 |

## Test Results

### no_std (default features)

```
test result: ok. 160 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

### std feature enabled

```
test result: ok. 163 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

## Known Limitations

1. OCEL 2.0 export uses internal representation; pm4py import not yet validated end-to-end
2. Receipt `topo_order` limited to 64 ops
3. Loop counter saturates at 255
4. Ring capacity 64; overflow beyond 64 events/tick is counted but not buffered
5. Branchless gate benchmark is Linux-perf instrumented; macOS measurement is manual cycle count only
6. No external substrate verification yet (90% target)

## Commands to Reproduce

```bash
git clone https://github.com/seanchatmangpt/bcinr
cd bcinr
cargo check -p bcinr-powl
cargo test -p bcinr-powl --lib
cargo test -p bcinr-powl --lib --features std
cargo bench -p bcinr-powl
```

## Next Ladder to 90%

- External substrate verification (gVisor/GitHub Actions)
- pm4py OCEL import validation
- Camunda baseline comparison
- Independent reproduction
