# POWL Compiler: Zero-Allocation & Branchless Compilation Strategy

Based on the source code of `crates/bcinr-powl/src/compiler.rs`, the POWL (Partially Ordered Workflow Language) compiler transforms abstract workflow models into an authoritative, heap-free execution tape.

In alignment with the `BCINR` deterministic substrate mandate, the compiler achieves execution and verification without heap allocation (`#![no_std]`, zero alloc) through the following mechanisms:

## 1. The Flat `PowlTape` Execution Model
Instead of representing the compiled graph as a heap-allocated tree or linked nodes (which would require pointers and dynamic memory), the AST is compiled into a **flat execution tape (`PowlTape`)**.
- The tape has a hard, compile-time bound of **64 slots**.
- Execution dependencies (edges) are encoded entirely as `u64` bitmasks (`pred_mask` and `succ_mask`).
- Every segment of the AST compilation returns an entry mask and an exit mask (`u64`). Wiring nodes simply applies bitwise OR operations to these masks.

## 2. Bit-Parallel Transitive Closure Reachability Validation (BP-TCRV)
The compiler requires strict verification (Phase 2: Reachability) to ensure no nodes are disconnected. This is traditionally done using a BFS/DFS queue (which requires heap allocation). The POWL compiler completely avoids heap allocation by using a **Bit-Parallel Roy-Warshall algorithm**:
- It allocates a strict, fixed-size matrix on the stack: `let mut r = [0u64; 64];`.
- The transitive closure is evaluated over exactly 64 iterations, which can be fully unrolled by the compiler.
- It is entirely deterministic and requires no dynamic memory.

## 3. Branchless Arithmetic (CC=1)
Following the Radon Law ($CC=1$), the BP-TCRV algorithm executes without a single data-dependent branch (`if`, `match`, or early `return`). It translates control flow into bitwise polynomials:
```rust
for k in 0..64 {
    let r_k = r[k];
    for i in 0..64 {
        // Extract the reachability bit (1 or 0)
        let can_reach_k = (r[i] >> k) & 1;
        // Generate a full-width mask: !0u64 if 1, 0u64 if 0
        let mask = 0u64.wrapping_sub(can_reach_k);
        // Apply the bitwise OR without conditional jumps
        r[i] |= r_k & mask;
    }
}
```

## 4. Separation of Slow Rail (Compilation) and Hot Path (Tape Execution)
As per the `AGENTS.md` laws, compilation itself belongs to the **slow rail**. The legacy recursive compiler parses `Box` and `Vec` AST nodes, and the `v2` compiler builds `BTreeMap` indices for provenance inversion. 

However, the artifact they produce—the `PowlTape`—is a strict `#[no_std]` 64-slot struct. Once compilation finishes, the resulting tape uses `0` heap allocations and guarantees deterministic, branchless transition execution suitable for the authoritative hot path.
