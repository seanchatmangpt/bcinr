# Branchless Insertion Sort in `bcinr`

In the `bcinr` deterministic substrate, the standard iterative/conditional insertion sort violates the $CC=1$ rule, as it relies on variable iteration (loops) and data-dependent branches (`if` conditions for swapping). To comply with the zero-branching ($CC=1$) and zero-allocation requirements, sorting is transformed from a sequential control flow algorithm into an **O(1) branchless mathematical predicate** using structural ranking.

Here is exactly how nested sorting logic is flattened without conditional swaps or variable loops, as implemented in `insertion_sort_branchless_fixed.rs`.

## 1. Fixed-Width Decomposition
Instead of operating on variable-length arrays, the algorithm sorts a packed 64-bit word (8 bytes). The input `val` is unconditionally split into an 8-element array:
```rust
let b = [
    val & 0xFF,
    (val >> 8) & 0xFF,
    // ... up to (val >> 56) & 0xFF
];
```

## 2. Comparison-as-Arithmetic Predicates
Swaps are entirely eliminated. Instead, the algorithm determines the final index (rank) of each element. Two mathematical predicates are used to compute this rank in constant time:

**Strict Less-Than:**
```rust
let lt = |x: u64, y: u64| -> u64 { (x < y) as u64 };
```
**Equality / Stability Tie-Breaker:**
To maintain a stable sort and prevent collisions when duplicate bytes exist, a stability predicate is used. It grants a higher rank (shifts to the right) if two values are equal but one appeared later in the original array.
```rust
let eqe = |j: usize, i: usize, x: u64, y: u64| -> u64 { ((x == y) as u64) & ((j < i) as u64) };
```

## 3. Fully Unrolled Structural Ranking
The rank (final position, from 0 to 7) of any byte at index `i` is deterministically computed by comparing it against all 8 elements in the array. This replaces nested `for` loops with a fully unrolled sum of the `lt` and `eqe` predicates:

```rust
let rank = |i: usize| -> u64 {
    lt(b[0], b[i]) + lt(b[1], b[i]) + /* ... */ + lt(b[7], b[i])
    + eqe(0, i, b[0], b[i]) + /* ... */ + eqe(7, i, b[7], b[i])
};
```
This guarantees exactly $16$ arithmetic operations per byte to determine its sorted index, completely independent of the input data. 

## 4. Reassembly via Masking and Shifts
Once every byte's target rank is calculated, the bytes are shifted directly into their final slots in a new 64-bit word and combined using bitwise `OR`:

```rust
(b[0] << (rank(0) * 8))
| (b[1] << (rank(1) * 8))
| (b[2] << (rank(2) * 8))
| (b[3] << (rank(3) * 8))
| (b[4] << (rank(4) * 8))
| (b[5] << (rank(5) * 8))
| (b[6] << (rank(6) * 8))
| (b[7] << (rank(7) * 8))
```

## Summary
The `bcinr` implementation of "insertion sort" Abandons the traditional shift-and-insert mechanics entirely. By interpreting booleans as arithmetic (`as u64`), it calculates each element's exact final rank independently and shifts them into place in a single straight-line, branchless expression. This perfectly aligns with the `von_neumann_bypass` architectural rule: *"Bit-parallel mechanics over byte-sequential control flow."*
