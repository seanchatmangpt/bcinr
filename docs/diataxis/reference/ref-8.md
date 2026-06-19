# Reference: `sketch` — Probabilistic Sketches

Module: `bcinr_logic::sketch` (`crates/bcinr-logic/src/sketch.rs`)

Branchless update primitives for sublinear-space frequency sketches. The
exported surface is the Count-Min Sketch update step. All functions are
`#[inline(always)]` and branchless.

## Functions

| Function | Signature |
|----------|-----------|
| `count_min_sketch_update` | `fn(table: &mut [u32], hash: u64, depth: usize, width: usize)` |

## `count_min_sketch_update`

Records one observation of an item (identified by its precomputed `hash`)
into a Count-Min Sketch backed by a flat `depth * width` counter array.

For each row `i` in `0..depth`:

```rust
let h   = (hash ^ (i as u64)).wrapping_mul(0x9E3779B185EBCA87); // mix per row
let idx = (h as usize) % width;                                 // column in row i
table[i * width + idx] = table[i * width + idx].saturating_add(1);
```

The per-row mixing constant `0x9E3779B185EBCA87` is the 64-bit golden-ratio
(Fibonacci) multiplier, used to derive `depth` independent column indices
from a single input hash. Counters use `saturating_add`, so a saturated
counter stays at `u32::MAX` rather than wrapping.

**Contract.**
- `table.len()` MUST be at least `depth * width`; indices range over
  `i*width + (h % width)`. A too-small `table` will panic on out-of-bounds
  indexing.
- `width` MUST be non-zero (used as a modulus; `% 0` panics).
- `hash` is caller-supplied; quality of the estimate depends on the hash
  being well-distributed.
- This is the **update** half only. Querying (taking the per-row minimum
  across the same column indices) is performed by the caller using the same
  index derivation.

## Integrity / oracle

The verification scaffold (reference + 3 counterfactual mutants) is in the
`#[cfg(test)]` module; there is no exported `*_phd_gate`. See `phd_gates.md`.

## Complexity

| Function | Time | Space |
|----------|------|-------|
| `count_min_sketch_update` | `O(depth)` | `O(1)` (writes in caller-owned `table`) |

Count-Min Sketch space is `O(depth * width)` counters total, independent of
the number of distinct items — the sublinear-space property. Estimates are
one-sided (never underestimate), with error controlled by `width` and
failure probability by `depth`.

## Cross-references

- Constant-time, allocation-free design rationale: `explanation/theory-9.md`.
- Saturating counters: `reference/ref-3.md` (`fix`), `reference/ref-2.md`
  (`int`).
