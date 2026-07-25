# Handling Iteration in the BCINR Deterministic Substrate

According to **Rule 13** of the `bcinr` constitution (`AGENTS.md`), unbounded execution loops (`while`, variable-bound `for` loops, `break`, `continue`) are strictly banned. The system must guarantee branchless, timing-invariant, $O(1)$ constant-time execution paths without loop backedges in the final machine code. 

To replace dynamic iteration, `bcinr` relies on three core mechanics: **compile-time fixed masked loops**, **macro unrolling**, and **reproducible code generation (Rule 21)**.

## 1. Compile-Time Fixed Loops with Branchless Masking

When processing variable-length data (like arrays or tapes), standard programs iterate exactly `len` times. In `bcinr`, this introduces unacceptable timing channels. Instead, the runtime iterates over the **maximum theoretical bound** (e.g., exactly 64 times) and uses branchless masking to safely "nullify" operations on elements beyond the dynamic boundary.

**Example from `bcinr-powl/src/compiler.rs`:**
```rust
// Fixed loop bound of 64 allows complete compiler unrolling.
for i in 0..64 {
    // 1 if i is within the dynamic tape_len, 0 otherwise
    let in_bounds = (i < tape_len) as u64; 
    // All 1s if valid (!0u64), all 0s if out of bounds (0u64)
    let bounds_mask = 0u64.wrapping_sub(in_bounds);

    // Apply the bounds_mask. If out of bounds, succs becomes 0, 
    // making the operation a harmless no-op. No `break` needed!
    let succs = tape.ops[i].succ_mask & bounds_mask;
    r[i] = succs | (1u64 << i);
}
```
This forces the compiler to unroll the code into a deterministic block of straight-line instructions. Rule 13 dictates that merely writing a fixed `for` loop in Rust is not enough: it must be proven to compile with **no loop backedge** during object-code inspection (Rule 20).

## 2. Macro and Explicit Unrolling for Sequential Logic

For algorithms that search or sort, `bcinr` replaces dynamic iteration with explicit or macro-based unrolling. This generates exactly the required number of steps sequentially, maintaining $CC=1$ cyclomatic complexity.

* **Explicit Algorithmic Unrolling:** In `unrolled_binary_search_u32.rs`, instead of a `while low < high` loop, the code unrolls 32 explicit `step(pos, bit)` evaluations. It tentatively checks each bit from 31 down to 0, branchlessly subtracting the bit back out using a subtraction mask if it overshoots the target.
* **Macro Unrolling:** Used for sorting networks, such as in `odd_even_merge_sort_16u32.rs`. A `macro_rules! ce!` (Compare & Exchange) is invoked in a hardcoded sequence to replace dynamic sorting loops with a branchless $O(1)$ sorting network.

## 3. The Generated-Code Law (Rule 21)

Dynamic iteration is often required for traversing complex graphs, parsing, or dynamic structural discovery. `bcinr` outlaws this in the hot path. **Rule 21** solves this by moving graph traversal and structural parsing into the **slow rail** during the build phase via generators.

A Python script (e.g., `generator.py`) consumes the dynamic metadata/ontology and emits pre-computed, flat, constant-size Rust primitives. 

**Example from `bcinr-cmca/src/generated/generalization.rs`:**
Complex RDF knowledge bases are distilled into static, fixed-size matrices of fixed-point numbers:
```rust
pub static OBJECT_REGISTRY: [PackedSemanticState; N] = [ ... ];
pub static LENS_REGISTRY: [LensSpec; Q] = [ ... ];
pub static LAMBDA: [[NonNegativeFixed; Q]; K] = [ ... ];
```
* **Elimination of Discovery Loops**: Because the structures are strictly fixed-size `N`, `Q`, and `K` at compile time, the hot path simply loops over these pre-defined bounds.
* **Reproducibility**: Under Rule 21, these generated artifacts must be completely deterministic (verified byte-identical on clean runs), explicitly hashed (`RDF_INPUT_DIGEST`), and cannot be hand-edited.
* **Auditability**: Generated code is not exempt from the constitution. It must pass the `bcinr-cheat-scanner`, be $CC=1$, and undergo assembly-level auditing for loop backedges and branches. 

## Summary

By combining maximum-bounded iterations with branchless masking, macro-unrolled execution structures, and offloading all dynamic graph discovery to build-time code generators, `bcinr` guarantees that the runtime operates strictly within a fixed $O(1)$ envelope. Every iteration path is deterministic, mathematically bounded, and structurally verified in the final compiled object code.
