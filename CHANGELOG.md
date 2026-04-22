# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [26.4.21] - 2026-04-21

> **Bits, not bytes. Truth, not data. One atom, `n` lawful worlds.**

### Added
  - `U1_8` — `U_{1,8}` — 8 atoms / 1 B — Place atom; type alias for `u8`.
  - `U1_64` — `U_{1,64}` — 64 atoms / 8 B — Cell; one `u64` register.
  - `U1_512` — `U_{1,512}` — 512 atoms / 64 B — Block; one L1 cache line, `[u64; 8]`.
  - `U1_4096` — `U_{1,4096}` — 4096 atoms / 512 B — Domain; half a 4 KiB page, `[u64; 64]`.
  - `U1Coord` — packed `(domain:3b, cell:3b, place:3b)` coordinate into `u16`.
  - Branchless `fire_cell_branchless`, `fire_block_cell_branchless`, `compute_cell_delta`, `compute_block_delta`, `compute_domain_delta`.
  - **Every public fn carries a doctest.**
- **Cross-profile alignment** documented: 64 × `U1_4096` (64 × 512 B) ≡ `U_{1,64³}` (32 KiB). The atom count composes; the kernel law does not change.

### Changed
- Bumped `bcinr`, `bcinr-core`, `bcinr-logic`, `bcinr-api`, `bcinr-bench` to `26.4.21`.
- `bcinr` crate deps updated with explicit `path =` to local workspace copies.

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
