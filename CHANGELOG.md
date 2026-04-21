# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [26.4.22] - 2026-04-21

> **UniverseOS UBit Completion: authority · persistence · dispatch · supervision · context**

### Added
- **UBitCapability** — bounded authority tokens for the U_{1,262144} substrate. `cap_admit(transition_id, cap_mask)` uses same admission polarity as `cell_admit` (0 = admitted, nonzero = denied). `UBitCapabilityTable` holds up to 64 capabilities.
- **UBitImage** — first-class snapshot/restore/fork of the active U_{1,262144} universe. Branchless checkpoint/restore/fork on `UniverseExecutor`. Size-asserted ≤ 33 KiB.
- **UBitScopePlanner** — branchless kernel dispatcher: Cell (1 word / T0), Sparse (2–16 / T1), Domain (17–64 / T1), Full (65+ / T2). Full is always T2 — never T1.
- **UBitSupervisor** — self-healing kernel. Observes `UDelta` emissions, detects repeated family failures / drift spikes / denial storms, emits `UInstruction::Recovery` (kind = 9) as lawful motion. 16-slot fixed-capacity ring, no heap.
- **UBitField** (`feature = "alloc"`) — the U_{1,16777216} / 2 MiB L2 operating field: 64 aligned U_{1,262144} planes. Plane roles: law (0), capability (1), expected (2), reward_good (3), reward_bad (4), policy (5), value (6), drift (7), projections (8–15), scenarios (16–31), checkpoints (48–55), custom (56–63).
- **Capability-gated admission variants**: `eval_cell_with_cap`, `eval_sparse_with_cap`, `eval_domain_with_cap`, `eval_full_with_cap`.
- **`UInstruction::Recovery`** (`UInstrKind::Recovery = 9`) — supervisor-emitted recovery instruction, does not propose state motion.
- **`UniverseExecutor` OS integration**: `execute_one_authorized`, `checkpoint`, `restore`, `fork`, `boot_with_registry`, `drain_recovery`.
- **OS-completion smoke test**: `universe_os_completes_authority_image_planner_supervisor_field` — proves the full `UInstruction → UBitScopePlanner → UBitCapability → transition → UDelta → receipt → UBitSupervisor → UBitImage fork/restore` loop.

### Changed
- Bumped `bcinr`, `bcinr-core`, `bcinr-logic`, `bcinr-api`, `bcinr-bench` to `26.4.22`.

### Notes
- **Naming constitution**: `UBit` prefix for all new public runtime types. Formal docs use $U_{1,n}$ notation. Existing committed names (`UniverseBlock`, `UniverseExecutor`, `UniverseScratch`, `UCoord`, `UDelta`, `UInstruction`, `TransitionReceipt`, `ActiveWordSet`, `DeltaTape`, `TransitionRegistry`) are unchanged.
- **OS seam complete**: authority (UBitCapability) + persistence (UBitImage) + dispatch (UBitScopePlanner) + supervision (UBitSupervisor) + context (UBitField).

## [26.4.21] - 2026-04-21

> **Bits, not bytes. Truth, not data. One atom, `n` lawful worlds.**

### Added
- **Universe1_n — bit-native operating substrate** (`patterns::universe1`). Formal: `U_{1,n} = 𝔹ⁿ`. One Boolean truth atom, repeated `n` times. The substrate is parametric in `n`; the following profiles are defined in this release:
  - `U1_8` — `U_{1,8}` — 8 atoms / 1 B — Place atom; type alias for `u8`.
  - `U1_64` — `U_{1,64}` — 64 atoms / 8 B — Cell; one `u64` register.
  - `U1_512` — `U_{1,512}` — 512 atoms / 64 B — Block; one L1 cache line, `[u64; 8]`.
  - `U1_4096` — `U_{1,4096}` — 4096 atoms / 512 B — Domain; half a 4 KiB page, `[u64; 64]`.
  - `U1Coord` — packed `(domain:3b, cell:3b, place:3b)` coordinate into `u16`.
  - `U1Receipt` — typed FNV-1a rolling receipt (reuses Universe64 FNV constants for cross-tier mixing).
  - Branchless `fire_cell_branchless`, `fire_block_cell_branchless`, `compute_cell_delta`, `compute_block_delta`, `compute_domain_delta`.
  - **Every public fn carries a doctest.**
- **Universe64 Wave A–E aggregate subsystems**: IndexPlane, MaskBank / GeometryCompiler, LawKernel, AdmissionEvaluator, BoundaryKernel, ConformanceKernel, DriftKernel, DeltaTape (append-only ring), DeltaBus (bounded fan-out), ProjectionUpdater + ProjectionCache, ReadyMask (per-transition scheduler bitmap), ScopePlanner, RLKernel + RewardTable, PopcountHistogram / FixedHistogram, ScenarioRunner (seeded rollouts), and `UniverseExecutor` (single-core boot + instruction loop). These occupy the `U_{1,64³}` (262,144-atom, 32 KiB) profile.
- **38 Criterion benchmarks total**: 31 in `bcinr-bench/benches/universe64_bench.rs` covering every Wave A–E operation across T0–T2 tiers; 7 in `bcinr-bench/benches/universe1_bench.rs` covering the `U1_64`, `U1_512`, `U1_4096` and receipt kernels.
- **Cross-profile alignment** documented: 64 × `U1_4096` (64 × 512 B) ≡ `U_{1,64³}` (32 KiB). The atom count composes; the kernel law does not change.

### Changed
- Bumped `bcinr`, `bcinr-core`, `bcinr-logic`, `bcinr-api`, `bcinr-bench` to `26.4.21`.
- `bcinr` crate deps updated with explicit `path =` to local workspace copies.
- `README.md` updated with Universe Substrates table and new version banner.

### Fixed
- **`test_publish_batch` race condition**: replaced shared `FIRE_COUNT` atomic in `delta_bus.rs` test scaffolding with per-test isolated atomics (`RM`, `MULTI`, `UNSUB`, `BATCH`), eliminating parallel-test interference. Full suite now reports 1910 unit tests + 370 doctests + 38 benches green.

### Notes
- **Naming ontology**: the `U1_n` convention is intentional. `1` means one Boolean truth atom; `n` means the atom count. The byte footprint is a consequence (`n/8`), never a name. Readers should not confuse `U1_n` with a 1-bit integer type — it is a substrate type parameterized by atom count. This is the first-pass public surface of a substrate that addresses bits directly; byte-packing vocabulary is deliberately rejected.

## [26.4.17] - 2026-04-17

### Added
- **Branchless Calculus ($\mathcal{B}$-Calculus)**: Formalized the library around branchless, time-invariant computational primitives.
- **Formal Thesis**: Added `thesis.pdf` providing the theoretical foundation and empirical validation for civilizational-scale coordination.
- **Diátaxis Documentation**: Implemented a complete documentation suite (Tutorials, How-To, Explanation, Reference) with over 40 technical documents.
- **Adversarial Hardening**: Introduced `PanicFreeAlloc` trait and refactored `clamp_u32` to return `Result` types for resilient error handling.
- **Cache-Line Alignment**: Enforced 64-byte alignment on memory arenas and DFA transition tables to eliminate false-sharing jitter.
- **Generic Primitives**: Introduced the `Branchless` trait to unify bitset, mask, and saturation arithmetic across multiple bit-widths.

### Changed
- **Versioning**: Promoted library to `v26.4.17` for release readiness.
- **SIMD Safety**: Replaced all `core::mem::transmute` usage with safe `core::ptr::copy_nonoverlapping` patterns.
- **Dependency Purge**: Removed `prettytable-rs` from core dependencies to ensure a zero-dependency foundation.
- **API Facade**: Refactored `bcinr-api` to use clean re-exports instead of recursive wrapper functions.

### Fixed
- Fixed broken intra-doc links in SIMD documentation.
- Resolved workspace-level dependency resolution issues for benchmarking tools.
- Corrected DFA index calculation boundaries in adversarial stress tests.
