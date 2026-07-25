# Semantic three-way integration report

## Merge basis

- Merge base: `3338f59ae5fd11f0f5e05115e2981f6daa8caef2`
- Recovery: `8e80292a425207636628c6a489bb9a11c6092208` (14 commits ahead)
- Main: `22945aff08f0d0194febec924c93c5f6a192a942` (218 commits ahead)

This branch is rooted exactly at the supplied main commit. No blanket ours/theirs merge was performed.

## Summary

Main remains authoritative for production infrastructure, current CI, dependency versions, algorithm fixes and file modes, PDDL v2, POWL v2, and execution-v2 receipts. Recovery-unique work is admitted only where it does not overwrite those foundations. Shared CMCA and POWL runtime files remain fenced for semantic reconciliation rather than receiving an unreviewed textual merge.

## File categorization

### Main only

The production PDDL v2 surface, including `causal_v2.rs`, `cognitive.rs`, `downstream.rs`, `embedded.rs`, `ground_v2.rs`, `problem_builder.rs`, `production.rs`, `production_capability.rs`, `semantic_features.rs`, `sexpr.rs`, `task.rs`, `workflow_cmd/`, tests, examples, and release documentation.

Main-only POWL v2 files include `powl2.rs`, `scheduler_v2.rs`, `process_rewrite.rs`, and `process_toolkit.rs`. Main-only receipt work includes `execution_v2.rs`. Current `.github/workflows/ci.yml` and `.github/workflows/miri.yml` are retained unchanged.

### Recovery only

Recovery-only governance, audit material, quarantine moves, CMCA authority-separation modules, AutoSelect modules, POWL AutoSelect/MAPE-K bridges, causal-buffer receipt work, and branch-only tests/documents are eligible for integration after their dependency closure is demonstrated.

This checkpoint imports the recovery-only CMCA artifact boundary, certification authority, and proposal authority modules without exporting them from `lib.rs`; therefore they cannot alter production behavior before the shared CMCA surface is reconciled.

### Both modified — semantic conflicts

- `AGENTS.md`
- `Cargo.toml`
- `Cargo.lock` (must be regenerated, never line-merged)
- `bcinr-bench/benches/missing_bench.rs`
- `bcinr/examples/{dfa_matching,parse_primitives,utf8_and_sketch}.rs`
- `bcinr/tests/e2e/**`
- `crates/bcinr-cmca/Cargo.toml`
- `crates/bcinr-cmca/src/{allocator,fixed,lib,lrc,observatory}.rs`
- `crates/bcinr-cmca/src/generated/**`
- overlapping `crates/bcinr-cmca/tests/**`
- existing autonomic primitives
- `crates/bcinr-pddl/{Cargo.toml,README.md,src/lib.rs}`
- shared POWL files: `admit.rs`, `compiler.rs`, `dispatcher.rs`, `enterprise.rs`, `lib.rs`, `ocel.rs`, `projection.rs`, `typestate.rs`
- shared receipt files: `Cargo.toml`, `src/lib.rs`, `src/replay.rs`

### Generated artifacts

- `crates/bcinr-cmca/src/generated/case_studies.rs`
- `crates/bcinr-cmca/src/generated/generalization.rs`
- `crates/bcinr-cmca/src/generated/stability_profile.rs`
- recovery `crates/bcinr-cmca/generated-artifact/**`

Generated Rust is not manually merged. Generator/ontology/manifest contracts must be reconciled first, followed by deterministic regeneration and digest verification.

### Mode-only or formatting-only

Recovery reports zero textual additions/deletions across most `crates/bcinr-logic/src/algorithms/**`, `playground/**`, UI fixtures, and several POWL/document paths. Main content and mode `100644` are authoritative unless a nonzero textual patch is observed. Executable mode is reserved for actual scripts.

### Rename

Recovery quarantines:

- `crates/bcinr-cmca/generator.py` → `crates/bcinr-cmca/quarantine/legacy-generator/generator.py`
- `crates/bcinr-cmca/ontology/cmca-rdf.ttl` → `crates/bcinr-cmca/quarantine/legacy-ontology/cmca-rdf.ttl`
- `crates/bcinr-cmca/ontology/generalization.ttl` → `crates/bcinr-cmca/quarantine/legacy-ontology/generalization.ttl`

### Delete

The integration must not carry `.github/workflows/pr11-merge-main.yml`, root `*.rlib`, `audit_results*`, `auditor_output.txt`, `test_output.log`, `test-mutants-output.log`, `scratch.py`, `scratch.rs`, test binaries, or temporary `fix_*.py` / `patch*.py` / `patch*.diff` / `wipe_bridges.py` helpers.

## Semantic conflict table

| Path | Main change | Recovery change | Recommended result | Confidence | Manual review |
|---|---|---|---|---|---|
| `AGENTS.md` | Current repository governance and production composition rules | Authority separation, standing/evidence, generated-artifact and independent-certification rules | Semantic precedence merge; preserve both without weakening main | High | Yes |
| `Cargo.toml` | Current workspace/dependency composition | Adds CMCA audit harness | Start from main; add harness only after build proof | High | Yes |
| `Cargo.lock` | Production PDDL/POWL dependency transition | Small CMCA/tool delta | Discard and regenerate once manifests close | High | Yes |
| `cmca/allocator.rs` | Later allocation/runtime corrections | Typed refusals, proposal/admission, certification and stability gates | Reconcile public types and transaction boundary; no wholesale selection | High | Yes |
| `cmca/fixed.rs` | Later numeric corrections | Expanded domain refusals, saturation and mutant enforcement | Merge by numeric law; verify division, saturation, widths, normalization and overflow | High | Yes |
| `cmca/src/generated/**` | Current generated tables | Recovery generated tables/artifact contracts | Merge generators, regenerate, verify manifests/digests | High | Yes |
| `cmca/lib.rs` | Current feature/module surface | New authority modules | Begin from main and expose recovery modules only after dependency closure | High | Yes |
| `cmca/lrc.rs`, `observatory.rs` | Later runtime behavior | Proposal and telemetry authority chain | Adapt recovery authority chain to current main APIs | Medium | Yes |
| overlapping CMCA tests | Corrected current oracles | Additional refusals and hostile mutants | Main structural baseline plus unique recovery assertions; remove empty/duplicate tests | High | Yes |
| existing autonomic primitives | Current APIs | AutoSelect integration assumptions | Keep main primitives; adapt recovery-only AutoSelect modules | High | Yes |
| PDDL shared files | Production in-crate parser and PDDL→POWL v2 | Narrow exports/integration | Keep main; reapply only unique exports; never restore `pddl = "0.2"` | High | Yes |
| POWL shared files | POWL v2 model/compiler/scheduler | AutoSelect and MAPE-K assumptions | Keep v2 foundation; port recovery bridges to v2 APIs | High | Yes |
| receipt shared files | execution-v2/replay | causal buffer and hostile mutants | Keep execution-v2; integrate recovery behavior afterward | High | Yes |

## Unresolved issues

1. Shared CMCA runtime files require law-level reconciliation and cannot be certified through repository API inspection alone.
2. Recovery-only authority modules currently depend on types in the unresolved shared CMCA files; they are intentionally not exported in this checkpoint.
3. AutoSelect and POWL bridge modules require adaptation to current main APIs.
4. The authoritative CMCA generator command must be identified before generated sources or `Cargo.lock` are changed.

## Validation

Local execution was unavailable because this environment has no direct GitHub network transport and no materialized checkout. Consequently:

- `cargo fmt --all -- --check`: **UNKNOWN**
- `cargo check --workspace --all-features`: **UNKNOWN**
- workspace and focused test commands: **UNKNOWN**

The imported Rust files are inert until explicitly added to the crate module surface. Existing main production behavior and CI remain unchanged.

## State

`PARTIAL_ALIVE`: exact graph and branch actuation are receipted; non-conflicting recovery modules are preserved; semantic runtime conflicts remain explicitly fenced for manual integration and executable verification.
