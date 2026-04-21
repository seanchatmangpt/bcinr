# bcinr — BranchlessCInRust (v26.4.21)

> **NOTICE: Final `bcinr` Release and Transition to `unibit`**
>
> `bcinr` v26.4.21 is the final release of this deterministic process-discovery engine architecture. The project has successfully proven the $CC=1$ Radon Law, the Zero-Allocation Boundary, and the K-Tier timing constitution ($T_1 \le 200 \text{ns}$).
>
> We are now transitioning this physics engine into a full bit-native operating substrate called **`unibit`**. 
>
> For details on the transition, the future of the `dteam` architecture, and what `unibit` entails, please see:
> [**bcinr to unibit: The Final Transition**](docs/bcinr/FINAL.md)

---

`bcinr` is a performance-first, research-grade systems library providing a principled calculus for branchless algorithmics. It is designed for high-performance, deterministic autonomic systems where predictable latency, memory-safety, and side-channel resilience are non-negotiable.

## Key Features

-   **Deterministic Latency:** All primitives are branchless ($O(1)$ constant time), eliminating pipeline stalls and side-channel timing risks.
-   **$\mathcal{B}$-Calculus Formalism:** Each primitive is mapped within a formal framework ensuring invariant-preserving state transitions.
-   **Hardware-Agnostic SIMD:** High-performance implementations for SSE4.2 with verified portable fallbacks for ARM Neon and WebAssembly.
-   **Zero-Dependency Core:** The logic layer is strictly `no_std` and has zero external dependencies for maximum supply-chain security.
-   **Adversarial Hardening:** Panic-free memory arenas and `Result`-based contracts for numerical stability.
-   **Universe1_n — Bit-Native Substrate:** `U_{1,n} = 𝔹ⁿ`. One Boolean truth atom × `n`. The atomic unit is a bit, not a byte. Bytes are a consequence (`n/8`), not a primitive.

## Universe1_n Substrate

The operating substrate is parametric in `n`. One kernel law, many profiles. `n` is the count of one-bit truth atoms; the byte footprint is always `n/8`.

| Type         | Formal       | Atoms (n)    | Bytes   | Role                                      |
|--------------|--------------|--------------|---------|-------------------------------------------|
| `U1_8`       | `U_{1,8}`    | 8            | 1       | Place atom (alias for `u8`)               |
| `U1_64`      | `U_{1,64}`   | 64           | 8       | Cell — one `u64` register                 |
| `U1_512`     | `U_{1,512}`  | 512          | 64      | Block — one L1 cache line                 |
| `U1_4096`    | `U_{1,4096}` | 4096         | 512     | Domain — half of a 4 KiB page             |
| `UniverseBlock` | `U_{1,64³}` | 262,144   | 32 KiB  | attention × truth field (L1 Data Plane)   |
| (conceptual) | `U_{1,64⁴}`  | 16,777,216   | 2 MiB   | meaning field (L2 plane stack)            |

**`U1_n` is not a bit-width integer type.** It is a substrate where the atomic unit is one Boolean truth coordinate. `n` is the atom count.

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

-   **[bcinr to unibit: The Final Transition](docs/bcinr/FINAL.md)**: Details on the successor to `bcinr`.
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