# Offline Cyclic Dependency Detection in BCINR

In the BCINR architecture, the execution dependency graph (e.g., `cmca:dependsOn` relations from the RDF ontology) must be strictly acyclic to guarantee deterministic, finite execution. However, the algorithm required to detect cycles—Kahn's Topological Sort—fundamentally violates the authoritative Hot Path runtime laws. Therefore, cyclic dependency detection and traversal are strictly relegated to the **Slow Rail** during ahead-of-time (AOT) compilation.

## Why Kahn's Algorithm is Inadmissible on the Hot Path

The authoritative Hot Path demands `#![no_std]`, zero heap allocation, and a strict Cyclomatic Complexity of 1 ($CC=1$). Kahn's algorithm breaches these absolute laws:
1. **Dynamic Memory Allocation:** It requires maintaining a dynamically sized queue (e.g., `Vec<usize>`) for tracking nodes with an in-degree of 0, violating the Zero Heap Allocation law.
2. **Data-Dependent Iteration:** The core logic runs on an unbounded `while let Some(u) = queue.pop()` loop whose termination depends on the runtime graph topology, directly violating both the $CC=1$ stricture and the prohibition on unbounded execution.

Because of this, Kahn's algorithm runs exclusively on the Slow Rail during the `generate` phase.

## The Two-Phase Verification Protocol

During compilation (e.g., in [compiler.rs](file:///Users/sac/bcinr/crates/bcinr-powl/src/compiler.rs)), the execution graph must pass a two-phase static analysis protocol:

### Phase 1: Kahn's Topological Sort (Cycle Detection)
The Slow Rail executes Kahn's algorithm over the graph to detect non-loop cycles:
1. **In-Degree Computation:** It computes the in-degree of all non-`LoopRedo` nodes, intentionally ignoring incoming back-edges from designated loop redo paths.
2. **BFS Traversal:** It seeds a queue with all nodes having an in-degree of 0, exploring the graph and decrementing the in-degrees of successors.
3. **Cycle Verification:** If the traversal terminates and the number of visited non-`LoopRedo` nodes is less than the total number of non-`LoopRedo` nodes on the tape, a structural cycle exists. This immediately triggers a typed refusal (`CompileError::Cycle`). A cycle can never resolve to a silent zero or a partial result.

### Phase 2: Bit-Parallel Reachability Validation (BP-TCRV)
Even if Kahn's algorithm passes, a malicious graph could contain disconnected, unreachable cycles whose in-degrees never drop to 0, allowing them to evade Kahn's queue entirely. To seal the topological proof, BCINR applies a deterministic, fully unrolled Roy-Warshall transitive closure algorithm. It mathematically proves branchlessly that every active non-`LoopRedo` node falls within the strict transitive closure of the `entry_mask`.

## Topological Flattening for the Hot Path

Once mathematically proven to be acyclic and fully reachable, the Slow Rail performs **Topological Mask Flattening**. 

Dynamic dependency chains (`cmca:dependsOn`) are not passed downstream. Instead, entities are mapped to fixed array indices and their dependencies are flattened into fixed-width C-ABI hardware bitmasks (typically `u64`):
- **`pred_mask`:** A bitmask representing the prerequisites that must complete before an entity can execute.
- **`succ_mask`:** A bitmask representing the downstream consequences of an entity.

These pre-calculated masks are emitted as static Rust IR in `cmca_generated.rs` and handed across the strict `Gamma_CMCA` boundary. This AOT translation allows the Hot Path to bypass all pointer-chasing and dynamic traversals, safely resolving dependencies entirely through constant-time, branchless SWAR (SIMD-Within-A-Register) bitwise instructions.
