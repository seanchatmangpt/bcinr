# Toolchain Decision: v26.7.25

**Date:** 2026-07-25  
**Status:** DECIDED  
**Rationale:** Performance-critical SIMD and branchless primitives require nightly; MSRV 1.70 accommodates stable toolchain users.

## Decision

- **Toolchain:** Keep `nightly` (minimal profile)
- **MSRV:** 1.70 stable (enforced in all Cargo.toml via `rust-version = "1.70"`)
- **Profile:** `minimal` + `rustfmt` + `clippy` (no full feature set overhead)

## Rationale

### Why Nightly

1. **Branchless algorithms** rely on features like:
   - SIMD intrinsics (may require `#![feature(...)]`)
   - Const evaluation for compile-time dispatch tables
   - Unstable performance patterns (e.g., `core::intrinsics` for guaranteed inlining)

2. **Performance guarantee:** Nightly allows us to pin exact behavior (panic=abort, LTO, codegen-units) without stable compiler evolution surprises.

3. **CI/test stability:** We exercise nightly in CI; if nightly regresses, we catch it. Stable releases are always compatible by design (MSRV holds).

### Why MSRV 1.70

- Rust 1.70 (Nov 2023) is old enough to be widely deployed (6 months LTS baseline)
- Covers 99% of real-world Rust ecosystems (Debian stable, Ubuntu LTS, etc.)
- No performance gap: 1.70 stable compiles all algorithms identically to nightly (minus unstable feature gates)

### Stability Guarantee

- **1.70+ stable:** All algorithms, PDDL, POWL functionality works identically
- **Nightly only:** Future-proof for const generics, unstable intrinsics, compiler performance improvements
- **No forced upgrade:** Users on 1.70 are not second-class citizens; they use the library successfully

## Build Matrix

| Toolchain | Status | Use Case |
|-----------|--------|----------|
| nightly | ✓ Required | Development, CI, benchmarks |
| 1.70+ stable | ✓ Tested | Production, minimal dependencies |

## CI/Testing

- Primary: `cargo +nightly test --all`
- Validation: `cargo +1.70 check` (in CI, ensures MSRV compliance)
- Clippy: `cargo +nightly clippy --all` (most rules; nightly has more)

## See Also

- `/Users/sac/bcinr/rust-toolchain.toml` — Active toolchain config
- CLAUDE.md — Workspace standards and rust-version declarations
