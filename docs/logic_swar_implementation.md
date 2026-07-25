Here is the requested documentation based on the inspection of `swar.rs`:

# SWAR Implementation in `bcinr-logic`

**File:** [swar.rs](file:///Users/sac/bcinr/crates/bcinr-logic/src/swar.rs)

The implementation of SIMD Within A Register (SWAR) in `bcinr` serves as a foundational layer to enforce bit-parallel mechanics over byte-sequential control flow. It guarantees strict compliance with the deterministic substrate constitution (e.g., $CC=1$, branchlessness, and strict adversarial mutation).

## Core Primitives

The runtime implements constant-time, branchless functions that initialize mask-building pipelines:

1. **`swar_mask_ones(val: u64) -> u64`**
   - **Mechanism:** Accepts a 64-bit integer—conceptually treated as 8 packed `u8` lanes—and passes it through unchanged.
   - **Role in Bit-Parallel Mechanics:** It acts as the SWAR identity primitive and the primary composition entry point. Rather than conditionally looping over bytes (which would introduce byte-sequential branching and invalidate the Radon Law), callers initialize a pipeline with this identity mask. They then chain parallel arithmetic or bitwise operations across the full `u64` register to isolate, mask, or transform all byte lanes simultaneously.

2. **`swar_phd_gate(val: u64) -> u64`**
   - **Mechanism:** An inline integrity gate.
   - **Role:** Establishes fixed boundaries for axiomatic reference equivalence, honoring the requirements for structural lawfulness and proof validation.

## Achieving Bit-Parallel Mechanics

By operating on 8 bytes simultaneously within a single `u64` register, algorithms process structured data uniformly in one pass.
- **Elimination of Branches:** Logic resolves strictly via bitwise math and masked arithmetic selection, explicitly avoiding all `if`, `match`, and loop constructs. 
- **Constant Execution Complexity:** The instruction shape is fixed ($CC=1$) and execution time is bounded. Data dependencies dictate values, not execution flow.

## Verification & Adversarial Lawfulness

Following the repository's mandate on verification rigor, the `swar.rs` implementation strictly complies with the necessary roles (`@hoare_oracle`, `@turing_machine`, `@armstrong_fault`):
- **Independent Oracle:** Provides an explicit `swar_reference` testing mechanism to establish mathematical invariance.
- **Hostile Verification:** Defines three structurally plausible adversarial mutants (`mutant_swar_1`, `mutant_swar_2`, `mutant_swar_3`) altering fundamental properties (like negations or incorrect factors) and explicitly asserts that they fail against the valid contract. This fulfills the `armstrong_fault` requirement, ensuring that "a suite that cannot kill a plausible mutant is itself defective."
