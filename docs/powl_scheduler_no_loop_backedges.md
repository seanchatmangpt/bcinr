# How the POWL VM Scheduler Avoids Loop Backedges (Rule 13)

Under **Rule 13** of the BCINR Deterministic Substrate Constitution, authoritative iteration must not contain loop backedges in the emitted object code. The POWL VM scheduler strictly adheres to this through a combination of compile-time monomorphization and fixed-bound compiler unrolling, primarily found in `crates/bcinr-powl/src/const_scheduler.rs`.

## 1. Compile-Time Topology Monomorphization (`const_tick`)
When the POWL graph topology is known at compile time, the scheduler delegates execution to the **const scheduler**. The dependency graph size (`N`) and the predecessor masks (`PREDS`) are passed as const generics:

```rust
pub fn const_tick<const N: usize, const PREDS: [u64; MAX_OPS]>(done: &mut u64) -> u64
```

Because `N` and `PREDS` are embedded in the type system, the compiler propagates predecessor masks as immediate constants, eliminating memory loads.

## 2. Ahead-of-Time Topological Pre-computation
The scheduler completely removes the need for dynamic graph traversal at runtime. A `const fn` named `topo_order` computes the topological firing order (`ConstTopology::ORDER`) during compilation. The runtime execution only performs bitwise evaluations in this precomputed order. 

## 3. Straight-Line Instruction Emission (No Backedges)
In `static_tick` (the function underlying `const_tick`), there is a bounded loop: `while i < n`. Because `n` is instantiated as the const generic `N` (typically small, e.g., `N <= 8`), the compiler is capable of **fully unrolling** this iteration into straight-line branchless arithmetic.

As documented in `const_scheduler.rs`, the compiler emits sequences of bitwise operations (e.g., `AND`, `SUBS`, `CSINV`, `ORRS` on ARM64) with **zero loop counters and zero backedge jumps**.

## 4. Statically Fixed Loop Bounds in Hot Paths
For other core routines where compile-time topology isn't available, iterations are strictly bounded by static constants (e.g., `0..64` for iterating over a `u64` bitmask). For example:

* **`ocel.rs` (Symmetric Run-Bounded Conformance Gating)**: The loop bound is explicitly set to `0..64`, allowing the compiler to generate unrolled assembly using conditional move instructions (`CSEL`/`CMOV`).
* **`compiler.rs` (`bp_tcrv_validate_reachability`)**: Replaces standard dynamic reachability checks with a bit-parallel Roy-Warshall algorithm limited strictly to 64 iterations, which is also completely unrolled by the compiler.
