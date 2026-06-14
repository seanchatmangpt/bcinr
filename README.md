# bcinr — BranchlessCInRust (v26.4.22 → v26.6.13)

`bcinr` is a performance-first, research-grade systems library providing a principled calculus for branchless algorithmics. It is designed for high-performance, deterministic autonomic systems where predictable latency, memory-safety, and side-channel resilience are non-negotiable.

> **Status:** Undergoing comprehensive gap-closure remediation (v26.6.13). See [Audit & Remediation](#audit--remediation) below.

## Key Features

-   **Deterministic Latency:** All primitives are branchless ($O(1)$ constant time), eliminating pipeline stalls and side-channel timing risks.
-   **$\mathcal{B}$-Calculus Formalism:** Each primitive is mapped within a formal framework ensuring invariant-preserving state transitions.
-   **Hardware-Agnostic SIMD:** High-performance implementations for SSE4.2 with verified portable fallbacks for ARM Neon and WebAssembly.
-   **Zero-Dependency Core:** The logic layer is strictly `no_std` and has zero external dependencies for maximum supply-chain security.
-   **Adversarial Hardening:** Panic-free memory arenas and `Result`-based contracts for numerical stability.

## Installation

Add `bcinr` to your `Cargo.toml`:

```toml
[dependencies]
bcinr-core = "26.4.22"  # v26.6.13 in active remediation
```

> **Note:** v26.6.13 is in active development. Significant gap-closure work is underway (see [Audit & Remediation](#audit--remediation)). Pin to `26.4.22` for stability.

## Quick Start

```rust
use bcinr_core::api::{select_u32, add_sat_u8, clamp_u32};

// Branchless selection: mask 0xFFFFFFFF selects first arg, 0x0 selects second
let val = select_u32(0xFFFFFFFF, 10, 20);
assert_eq!(val, 10);

// Saturating arithmetic: never overflows
let sum = add_sat_u8(200, 100);
assert_eq!(sum, 255);

// Safe clamping: returns Result for contract validation
let clamped = clamp_u32(150, 0, 100).unwrap();
assert_eq!(clamped, 100);
```

## Documentation (Diátaxis)

The documentation is organized to support different stages of integration and research:

-   **[Tutorials](docs/diataxis/tutorials/)**: Walkthroughs for implementing kernels and SIMD vectorization.
-   **[How-To Guides](docs/diataxis/how-to/)**: Practical solutions for side-channel hardening and WCET bounding.
-   **[Explanations](docs/diataxis/explanation/)**: Deep-dives into the Branchless Calculus and architectural design.
-   **[References](docs/diataxis/reference/)**: Full API catalog and technical specifications.
-   **[Anti-Patterns](docs/diataxis/explanation/anti-patterns.md)**: Critical pitfalls and structural hazards to avoid.

[Full Documentation Index](docs/diataxis/INDEX.md)

## Audit & Remediation

### Recent Work (v26.6.13 Initiative)

Over the past week, a comprehensive audit of the codebase was conducted to identify and systematize remediation efforts:

**Audit Findings:**
- **1,049 systematic cheats** detected across 308 algorithm files
- **5 cheat patterns** identified and automated for detection:
  - **Padding Boilerplate:** 275 files with artificial length-inflation comments
  - **Fake Hoare Proofs:** 265 files with copy-pasted verification claims
  - **Circular References:** 100+ files where test references are identical to implementations
  - **Magic Constants:** 37+ files with hardcoded `0xDEADBEEF`, `0xCAFEBABE` in production
  - **Self-Canceling XOR:** 50+ files with logic-erasing expressions (`A ^ A`)

**New Quality Assurance Tool:**
- **`bcinr-cheat-scanner`** — Rust binary detects all 5 cheat patterns in <1 second
- Integrated into CI pipeline: `cargo make scan-cheats` blocks commits with cheats
- Machine-readable output (`CHEAT[TYPE]: path:line — reason`)

**Remediation Phases (v26.6.13):**
1. ✅ **Audit Complete** — 10-agent analysis delivered detailed findings
2. 🔄 **Phase 1-5 (In Progress)** — Compilation fixes, boilerplate removal, safety hardening
3. 📋 **Phase 6-8 (Planned)** — Algorithm rewrites, oversimplified tier 201-300 redesign
4. 🚀 **Phase 9-10 (Planned)** — Versioning, release notes, v26.6.13 ship

**See Also:**
- [Detailed Audit Report](AUDIT.md) *(to be generated)*
- [Remediation Roadmap](REMEDIATION.md) *(to be generated)*
- Commit: `ebc6121` (cheat-scanner tool introduction)

## Performance & Architecture

-   **[Benchmark Charter](docs/BENCHMARKS.md)**: Performance, memory, and complexity targets.
-   **[Architecture Overview](ARCHITECTURE.md)**: Domain taxonomy and design philosophy.

## Development & Testing

### Quality Gates

The project includes automated gates to prevent regressions:

```bash
# Scan for systematic cheats (padding, fake proofs, circular refs, magic constants)
cargo make scan-cheats

# Validate branchless contract compliance
cargo make contract-gate

# Full CI pipeline (runs gates, tests, linters, audits)
cargo make ci
```

The cheat scanner is **non-optional** in CI — commits are blocked if any cheat patterns are introduced.

## Formal Basis

For the formal mathematical proof and civilizational-scale analysis of this library, see the academic thesis:
[**Formal Verification of Deterministic Substrates: The $\mathcal{B}$-Calculus for Civilizational-Scale Irreversible Systems**](./thesis.pdf).

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
