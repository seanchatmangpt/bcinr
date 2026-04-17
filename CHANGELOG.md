# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
