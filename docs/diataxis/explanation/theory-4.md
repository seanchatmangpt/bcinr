# SWAR and Data Parallelism Without SIMD

SWAR — **S**IMD **W**ithin **A** **R**egister — is the technique of treating
one ordinary integer register as a vector of smaller lanes and operating on
all lanes at once with plain arithmetic and bitwise instructions. It is how
bcinr extracts data parallelism in a `no_std`, portable core that cannot
assume any particular SIMD instruction set. This document explains the idea
and the two hazards that make it subtle.

## One register, many lanes

A `u64` is sixty-four bits. If your data is bytes, that register holds eight
of them. A bitwise op on the `u64` is, simultaneously, that op on all eight
bytes — because AND, OR, XOR, and shift have no carry between bit positions.
`is_ascii_u64_slice` in `scan.rs` is the clean example:

```rust
accumulator |= val & 0x8080_8080_8080_8080;   // test bit 7 of all 8 bytes at once
...
accumulator == 0                              // ASCII iff no high bit anywhere
```

One `&` examines eight bytes. Eight-way parallelism, zero SIMD, runs
anywhere a `u64` exists.

## The carry problem, and how lane masks solve it

Bitwise ops are lane-safe for free. *Arithmetic* is not: an `add` propagates
carry across the whole register, so a carry out of lane 0 corrupts lane 1.
SWAR addition handles this by masking the lanes into two interleaved groups
so carries have nowhere to spill, adding each group, then recombining — the
classic "SWAR add" decomposition. The same hazard shapes the reductions in
`reduce.rs`. `horizontal_sum_u8x8` sums eight bytes pairwise into wider
fields at each step so that intermediate sums cannot overflow their lane:

```
  step 1: pair bytes  -> 4 x 16-bit partial sums   (mask 0x00FF00FF00FF00FF)
  step 2: pair those  -> 2 x 32-bit partial sums   (mask 0x0000FFFF0000FFFF)
  step 3: pair those  -> 1 x 64-bit total          (mask 0x00000000FFFFFFFF)
```

Each widening step doubles the headroom *before* the add, so no lane ever
carries into its neighbour. The reduction tree is `log2(lanes)` deep — three
steps for eight lanes — which is the SWAR analogue of a parallel reduction.

## Broadcast and compare as building blocks

Two idioms recur. **Broadcast** replicates a scalar into every lane so it
can be combined lane-wise; the multiplications by `0x0101010101010101` in
`horizontal_max_u8x8`/`horizontal_min_u8x8` are exactly this — smearing a
byte across all eight positions. **Lane compare** turns a per-lane predicate
into a per-lane mask using the high-bit-of-each-lane trick (the `>> 7` and
`& 0x0101...` in those same reductions), which is just the mask encoding of
`theory-3.md` applied independently inside each lane.

## SWAR versus true SIMD

When a real vector unit is available, bcinr's `simd.rs` models the 128-bit
operations (`splat_u8x16`, `shuffle_u8x16`, `movemask_u8x16`) that a target
like SSE4.2 would lower to single instructions. The relationship is a
hierarchy of *fallbacks*, not a competition:

```
  preferred:  SSE4.2 / NEON intrinsic   (16 lanes, 1 instruction)
  portable:   SWAR over u64             (8 lanes, a few instructions)
  scalar:     per-element loop          (1 lane, always correct)
```

The portable core stays SWAR so it compiles for ARM, WebAssembly, and bare
metal alike; the SIMD layer is an acceleration that must produce *bit-identical*
results, which is why `simd.rs` carries the same reference-oracle test
scaffold as every other module.

## Why SWAR fits the library's goals

SWAR parallelism is branchless by construction — it processes whole chunks
with straight-line bitwise/arithmetic ops — so it inherits the time
invariance of `theory-1.md` while delivering throughput. It needs no
runtime feature detection, no intrinsics, and no allocator, which keeps it
inside the `no_std`, zero-dependency boundary described in `theory-9.md`.
The cost is real but bounded: the per-lane parallelism is narrower than a
vector unit's, and the masking adds a few operations, which is the trade
analysed in `theory-2.md`.
