# POWL Compilation: Single-Pass Topological Sort and Verification Bounds

This document outlines the theoretical constraints and architectural mechanics underlying the Single-Pass Topological Sort rule for compiling Partially Ordered Workflow Language (POWL) graphs in the BCINR substrate.

## The Admittance of Kahn's Algorithm on the Slow Rail

To execute safely within BCINR's execution pipelines, POWL graphs must be structurally proven to be free of non-loop cycles prior to execution. We enforce this through Kahn's Topological Sort. However, Kahn's algorithm fundamentally violates the authoritative Hot Path runtime laws and is therefore strictly mandated to the **Slow Rail**.

### Why Kahn's is Inadmissible on the Hot Path

Kahn's algorithm relies on dynamic data structures and unbounded iteration:
1. **Dynamic Queue Allocation**: It maintains a dynamically sized queue (e.g., `let mut queue: Vec<usize>`) to track nodes with an in-degree of 0. This violates BCINR's absolute **Zero Heap Allocation** law.
2. **Data-Dependent Loop Termination**: The core logic runs on a `while let Some(u) = queue.pop()` loop. The iteration count and loop termination depend entirely on the incoming graph topology. This violates the **No Unbounded Execution** and **$CC=1$** (Cyclomatic Complexity = 1) strictures.

Because the Hot Path requires a purely branchless, deterministic instruction shape (`#![no_std]`, no alloc, $CC=1$), Kahn's algorithm is restricted to the ahead-of-time compiler rail. It operates during AST compilation to mathematically verify the topological structure before generating the branchless $O(1)$ SWAR execution tape. 

## Bit-Parallel Reachability and Cycle Absence Proof

While Kahn's algorithm ensures that the graph does not contain topological cycles among its visited nodes (by validating that the total number of visited non-LoopRedo nodes equals the graph's total non-LoopRedo node count), BCINR relies on a supplementary **Bit-Parallel Transitive Closure Reachability Validation (BP-TCRV)** to mathematically seal the topological proof ahead-of-time.

### Mathematical Proof via BP-TCRV

The bit-parallel reachability pass utilizes a deterministic, fully unrolled Roy-Warshall algorithm. It mathematically guarantees the absence of disconnected cyclic components and unreachable execution paths in constant time:

1. **Unrolled Matrix Initialization**: A $64 \times 64$ reachability matrix $R$ is initialized branchlessly using the explicit `succ_mask` of each tape slot.
2. **Branchless Transitive Closure**: The algorithm executes exactly 64 constant-time iterations. For each pivot $k \in [0, 63]$, the reachability is updated purely algebraically:
   $$R_i^{(k+1)} = R_i^{(k)} \cup (R_k^{(k)} \text{ if } k \in R_i^{(k)})$$
   Translated into SWAR instructions:
   ```rust
   let can_reach_k = (r[i] >> k) & 1;
   let mask = 0u64.wrapping_sub(can_reach_k);
   r[i] |= r_k & mask;
   ```
3. **Validating the Reachable Set**: The algorithm computes the strict union of all nodes structurally reachable from the `entry_mask`.

By establishing the total transitive closure without conditional branches, the BP-TCRV explicitly proves that **every active non-`LoopRedo` operation is strictly bounded and reachable from the tape's entry mask**. 

If a malicious graph contained an unreachable cycle, the nodes within that cycle would have in-degrees that never reach zero (evading Kahn's queue) while also failing the BP-TCRV mask check. By intersecting Kahn's invariant (the exact required node count must be successfully popped off the topological frontier) with the BP-TCRV invariant (all active nodes must fall within the transitive closure of the entry nodes), the Slow Rail statically and mathematically proves the absolute absence of both reachable and unreachable non-loop cycles ahead-of-time, ensuring a flawless directed acyclic topology for the branchless Hot Path scheduler.
