# bcinr — BranchlessCInRust (v26.4.21)

`bcinr` is a performance-first, research-grade systems library providing a principled calculus for branchless algorithmics. It is designed for high-performance, deterministic autonomic systems where predictable latency, memory-safety, and side-channel resilience are non-negotiable.

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
bcinr-core = "26.4.21"
```

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

## Performance & Architecture

-   **[Benchmark Charter](docs/BENCHMARKS.md)**: Performance, memory, and complexity targets.
-   **[Architecture Overview](ARCHITECTURE.md)**: Domain taxonomy and design philosophy.

## Formal Basis

For the formal mathematical proof and civilizational-scale analysis of this library, see the academic thesis:
[**Formal Verification of Deterministic Substrates: The $\mathcal{B}$-Calculus for Civilizational-Scale Irreversible Systems**](./thesis.pdf).

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
