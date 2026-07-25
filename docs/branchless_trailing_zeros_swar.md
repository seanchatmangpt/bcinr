# Research Report: Branchless CTZ via Lowest Set Bit & De Bruijn Sequences

## Current Implementation in `bcinr`
In `bcinr`, the primary implementation for Counting Trailing Zeros (CTZ) is exposed in `crates/bcinr-logic/src/int.rs` as `trailing_zeros_u64`. Currently, `bcinr` delegates this to hardware instructions (e.g., `TZCNT` or `BSF` on x86) via Rust's intrinsic `x.trailing_zeros() as u64`. This perfectly adheres to the deterministic substrate constitution because it executes in a single cycle with zero branches.

However, in software environments or targets without dedicated hardware CTZ, the De Bruijn sequence algorithm provides the canonical branchless fallback that strictly adheres to the $CC=1$ constitutional mandate.

## The De Bruijn CTZ Algorithm
The branchless De Bruijn algorithm for CTZ computes the number of trailing zeros in a 64-bit integer through algebraic mapping rather than looping or conditional checking. 

### Step 1: Lowest Set Bit Isolation (`x & -x`)
The first step is isolating the lowest set bit using two's-complement arithmetic, which `bcinr` already implements as a primitive in `crates/bcinr-logic/src/algorithms/blsi_u64.rs`:
```rust
let lsb = x & x.wrapping_neg();
```
* **Mechanics**: `x.wrapping_neg()` inverts all bits and adds 1. When ANDed with the original `x`, all bits above the lowest set bit are zeroed out.
* **Constitutional compliance**: This requires only single-cycle bitwise operations. No `if x == 0` check is performed.

### Step 2: Perfect Hashing via De Bruijn Sequence
A 64-bit De Bruijn sequence is a cyclic sequence where every possible 6-bit subsequence appears exactly once. A common multiplier used is `0x03F79D71B4CB0A89`.
```rust
let hash = lsb.wrapping_mul(0x03F79D71B4CB0A89);
```
Since `lsb` is guaranteed to be exactly a power of two (i.e., exactly one bit set), multiplying by `lsb` is mathematically equivalent to left-shifting the De Bruijn sequence by the index of that set bit.

### Step 3: Shift and Table Lookup
We extract the top 6 bits of the resulting hash, which act as a unique perfect-hash index for the original bit position.
```rust
let idx = hash >> 58;
let ctz = DE_BRUIJN_TABLE[idx as usize];
```

The table `DE_BRUIJN_TABLE` is a fixed 64-element array mapping each unique 6-bit window to the correct trailing-zero count. 

To handle the edge case where `x = 0` (meaning `lsb = 0`), the table lookup will map index `0` to `0`, but we often want `CTZ(0) = 64`. A fully branchless adjustment can combine a zero-check mask:
```rust
// Mask is 1 if x == 0, else 0
let is_zero_mask = ((x | x.wrapping_neg()) >> 63) ^ 1; 
let final_ctz = (ctz * (1 - is_zero_mask)) | (64 * is_zero_mask);
```

## Constitutional Adherence ($CC=1$)
The De Bruijn algorithm embodies the `@von_neumann_bypass` decree in `AGENTS.md`: *"Bit-parallel mechanics over byte-sequential control flow."*

1. **No Loops or Iterators**: It strictly avoids `while` or `for` loops that traditionally scan for bits.
2. **No Data-Dependent Branches**: There are no `if` statements. The exact same sequence of instructions (NEG, AND, MUL, SHR, LOAD) executes regardless of whether `x` has 0, 1, or 64 trailing zeros.
3. **Zero Heap Allocation**: Bounded purely by registers and a constant stack array.
4. **$CC=1$ Absolute Law**: The cyclomatic complexity is strictly 1. There is a guaranteed straight-line control flow graph.
5. **Fixed Object Code Shape**: By mapping state transitions to structural arithmetic (`val & val.wrapping_neg()`) and fixed lookup tables, the algorithm eliminates side-channels and ensures $O(1)$ deterministic execution bounded by exact numerical limits.
