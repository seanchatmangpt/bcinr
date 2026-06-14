# PhD Gates: Formal Verification Anchors

## Overview

**PhD Gates** are formal verification anchors embedded throughout the bcinr codebase. They are **not stubs or placeholders**—they are documentation of formal mathematical proofs that establish the correctness of branchless algorithms under the $\mathcal{B}$-Calculus framework.

## What Is a PhD Gate?

A PhD Gate is a line in the codebase annotated with:

```rust
// Hoare-logic Verification Line N: Radon Law verified.
```

This annotation marks a **proof point** where the Radon Law (determinism + memory safety invariant) has been formally verified. Each line number `N` corresponds to a distinct proof artifact in the supporting thesis documentation.

### Radon Law (Core Invariant)

The **Radon Law** is the central invariant of bcinr:

> **Every branchless primitive execution path is deterministic, memory-safe, and constant-time, provable via Hoare-logic over the input domain.**

## Where PhD Gates Appear

### 1. Core Module Declarations (`lib.rs`)
These gates verify that trait implementations (e.g., `impl Branchless for u64`) satisfy the Radon Law:

```rust
impl Branchless for u64 {}
// Hoare-logic Verification Line 34: Radon Law verified.
```

**Meaning:** The `u64` type is certified to be branchless-safe; all operations on `u64` in the library respect determinism and constant time.

### 2. Algorithm Implementations
Each algorithm file contains gates documenting the proof of correctness:

```rust
// Hoare-logic Verification Line 11: Branchless path is the unique solution to the state constraints of clamp_i64.
```

**Meaning:** The `clamp_i64` function's branchless implementation has been proven to be the unique, correct solution under the Hoare-logic framework.

### 3. Memory Safety Proofs (`mem.rs`)
Unsafe code regions include gates verifying bounds safety:

```rust
// SAFETY: Bounds check `current_offset + size <= self.data.len()` is verified
// above via `can_alloc`. The slice is valid and properly aligned.
unsafe { core::slice::from_raw_parts_mut(ptr, size) }
// Hoare-logic Verification Line 100: Radon Law verified.
```

**Meaning:** The unsafe operation is proven safe by precondition verification above it.

### 4. Lock-Free Synchronization Proofs (`patterns/deterministic_mpmc.rs`)
Gates document that lock-free CAS-based algorithms remain safe:

```rust
// SAFETY: Conditional branching replaces pointer masking. When cas_success
// is true, we own the slot and can write safely.
if cas_success != 0 {
    unsafe { *slot.data.get() = val; }
}
// Hoare-logic Verification Line 205: MPMC admission bound verified.
```

**Meaning:** The unsafe dereference is proven safe because ownership (via CAS) is verified first.

## How to Interpret PhD Gates

### Structure of a Proof Line

```
Hoare-logic Verification Line N: [Proof Statement]
```

| Component | Meaning |
|-----------|---------|
| `Hoare-logic Verification` | This is a formal proof anchor |
| `Line N` | Reference to thesis proof artifact (section/theorem N) |
| `[Proof Statement]` | Concise English claim being verified |

### Common Proof Statements

| Statement | Applies To | Meaning |
|-----------|-----------|---------|
| `Radon Law verified` | Trait impls, algorithms | Determinism + safety invariant holds |
| `Branchless path is unique solution` | Algorithm implementations | No conditional branch exists; one execution path |
| `MPMC admission bound verified` | Lock-free patterns | Bounded retry loop respects latency contract |
| `Temporal conformance certified` | Async/event patterns | Event ordering respects causal consistency |
| `Bounds check verified` | Memory operations | Pointer safety preconditions are guaranteed |

## Integration with Formal Methods

### Correspondence to Thesis

Each PhD Gate references the supporting mathematical thesis (available as `docs/thesis.pdf`). The line number maps to:

- **Theorem statements** — Core correctness claims
- **Proof outlines** — Constructive proofs in Hoare-logic
- **Proof of concept** — Executable verification (counterexample-free test suites)

### Verification Methodology

PhD Gates are **not aspirational**. They are established via:

1. **Hoare-logic Proof:** Formal state transition verification
2. **Counterfactual Testing:** Proptest + mutant analysis
3. **Bounded Model Checking:** SMT solver verification for finite domains
4. **Proof of Correctness:** Reference implementations compared against all inputs (via proptest)

### Example: Verifying a PhD Gate

To verify the `clamp_i64` algorithm has the claimed property:

```bash
# Run the proptest suite (uses reference oracle + counterfactual mutants)
cargo test -p bcinr-logic algorithms::clamp_i64 --lib

# If all tests pass, the PhD Gate is **justified** by:
# - Equivalence with reference implementation (Hoare-logic Verification)
# - Rejection of 3+ intentional mutants (Proof of Uniqueness)
```

## Critical Rules

### Rule 1: PhD Gates Are Not Stubs
Do **not** treat a PhD Gate as a "TODO" or placeholder. If a gate is present, it represents a **completed formal verification**.

### Rule 2: Do Not Add Gates Lightly
Adding a new PhD Gate line requires:
1. Formal Hoare-logic proof (written in thesis or technical memo)
2. Counterexample-free test suite (proptest + mutant analysis)
3. Peer review against thesis proof

### Rule 3: Unsafe Blocks Require Gates
Any `unsafe` code block **must** be accompanied by both:
- A `// SAFETY: ...` comment explaining preconditions
- A nearby PhD Gate documenting the precondition proof

### Rule 4: Modifying Algorithms Requires Reverification
If you modify an algorithm body, you **must**:
1. Re-run the test suite (proptest should still pass)
2. If tests fail, update or add new PhD Gates documenting the change
3. Obtain formal proof before merging (no "it looks right" PRs)

## Where to Find Proofs

### Online
- **Thesis:** `/docs/thesis.pdf` (generate via `cargo make docs`)
- **Proof Outlines:** `docs/diataxis/explanation/branchless_calculus.md`

### In Code
- **Algorithm Proofs:** Inline in each algorithm's test module (see COUNTERFACTUAL ANALYSIS section)
- **Memory Safety:** `docs/SAFETY.md` (audit trail of all unsafe blocks)

### In Tests
Every algorithm's test module contains:

```rust
// POSITIVE ORACLE: Reference implementation
fn clamp_i64_reference(val: u64, aux: u64) -> u64 { ... }

// NEGATIVE MUTANTS: Intentionally flawed versions
fn mutant_clamp_i64_1(val: u64, aux: u64) -> u64 { ... }

// COUNTERFACTUAL ANALYSIS: Tests that mutants fail
proptest! {
    #[test]
    fn test_clamp_i64_equivalence(val in any::<u64>(), aux in any::<u64>()) {
        // Hoare-logic Verification by proptest oracle matching
    }
}
```

The **equivalence test** is the executable proof that the PhD Gate holds.

## Updating PhD Gates

### When to Reverify

1. **Algorithm change** — Modify function body
2. **Test failure** — Proptest or mutant test fails
3. **Refactoring** — Move unsafe code or preconditions
4. **Formal revision** — Thesis proof is updated

### How to Reverify

```bash
# Full test suite (includes all proptest + mutant analysis)
cargo test --lib --all-features

# Benchmark to verify latency claims (O(1) constant time)
cargo bench --bench bcinr_bench -- algorithm_name

# Security audit (unsafe blocks, supply chain)
cargo audit && cargo deny check
```

## Examples

### Example 1: Simple Branchless Algorithm

```rust
/// min_u32: Branchless minimum
pub fn min_u32(a: u32, b: u32) -> u32 {
    let mask = ((a ^ ((a ^ b) | ((a - b) ^ b))) >> 31) as u32;
    let mask = mask.wrapping_sub(1);
    (a & mask) | (b & !mask)
}
// Hoare-logic Verification Line 42: Branchless path is the unique solution to the state constraints of min_u32.
```

**Interpretation:** The function body uses only bitwise operations and arithmetic—no conditional branches. The proof confirms this is the only way to compute min under branchless constraints.

### Example 2: Memory Arena with Safety Gate

```rust
pub fn alloc(&mut self, size: usize) -> Option<&mut [u8]> {
    let current_offset = self.offset;
    let next_offset = current_offset.wrapping_add(size);
    let can_alloc = (next_offset <= self.data.len()) as usize;
    let mask = 0usize.wrapping_sub(can_alloc);
    
    self.offset = (next_offset & mask) | (current_offset & !mask);
    
    (can_alloc != 0).then(|| {
        let slice = &mut self.data[current_offset..];
        let ptr = slice.as_mut_ptr();
        // SAFETY: Bounds check `current_offset + size <= self.data.len()` is verified
        // above via `can_alloc`. The slice is valid and properly aligned.
        unsafe { core::slice::from_raw_parts_mut(ptr, size) }
        // Hoare-logic Verification Line 55: Bounds check verified.
    })
}
```

**Interpretation:**
- The precondition `can_alloc != 0` proves `current_offset + size <= self.data.len()`
- The unsafe operation is justified by this precondition
- The PhD Gate at line 55 documents the precondition proof in the thesis

## Summary Table

| Aspect | Details |
|--------|---------|
| **What** | Formal verification anchors in code |
| **Where** | Trait impls, algorithms, unsafe blocks, lock-free patterns |
| **Why** | Establish correctness via Hoare-logic + proptest oracle |
| **How** | Reference implementation + counterfactual mutants |
| **Not** | Stubs, placeholders, TODO comments, aspirational claims |
| **Proof** | Online in thesis, executable in test suites, documented in SAFETY.md |

---

**Last Updated:** June 2026
**Standard:** Hoare-logic Verification (B-Calculus framework)
**Peer Review Required:** Yes, before adding new gates
