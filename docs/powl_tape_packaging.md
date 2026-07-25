Here is the documentation detailing how the Slow Rail packs the validated graph into the fixed-width tape format for CC=1 execution:

# POWL Execution Tape Compilation & Validation (Slow Rail)

The Slow Rail is responsible for taking a Partially Ordered Workflow Language (POWL) AST (or v2 IR model) and compiling it into a flat, deterministic, fixed-width `PowlTape` array. This process eliminates complex graph traversal overhead and branches, preparing the data for bounded, $CC=1$ (branchless) Hot Path execution.

## 1. The Fixed-Width Tape Format

Instead of allocating a node graph on the heap with pointers, the entire graph is packed into a flat `PowlTape` array.

- **`PowlTape`**: A fixed-capacity structure containing an array of operations. The standard capacity is capped at 64 slots (`[Powl64Op; 64]`), which allows the Hot Path to represent entire states as single `u64` bitmasks. For larger workloads, a `PowlTapeLarge` (up to 512 operations, using `[u64; 8]` masks) is provided.
- **`Powl64Op`**: A single operation slot on the tape. 
  - In v2, this struct is strictly padded and aligned to exactly 64 bytes (`#[repr(C, align(64))]`) to map perfectly to a single CPU cache line.
  - It tracks logic solely through bitmasks:
    - `pred_mask`: A bitmask of predecessor slot indices that must complete before this op is enabled.
    - `succ_mask`: A bitmask of successor slot indices to activate upon completion.
    - `op_kind`: Semantic type (e.g., Activity, Silent, XorChoice, Loop).
    
Because the structure is an array with explicit bitmask edges, the Hot Path can use branchless bitwise logic to evaluate ready-states.

## 2. Compilation and Wiring (AST to Flat Tape)

The compiler uses a single recursive descent over the AST to allocate sequentially into the tape array. 

Each compiled AST node produces a `Segment` consisting of an **entry mask** (slots that start the execution of the subtree) and an **exit mask** (slots that terminate the execution). 

The compiler uses a `wire()` helper function to link operations: it sets the `succ_mask` of the previous exits to point to the next entries, and sets the `pred_mask` of the next entries to require the previous exits. 

## 3. Two-Phase Graph Validation Protocol

After packing the operations onto the tape, the Slow Rail enforces two safety properties before the tape is considered valid for the $CC=1$ Hot Path.

### Phase 1: Kahn's Cycle Detection
The compiler computes in-degrees for all non-`LoopRedo` nodes (ignoring back-edges) and performs a topological Breadth-First Search. If the BFS visits fewer non-redo nodes than are allocated on the tape, it proves a non-loop cycle exists, and compilation is aborted (`CompileError::Cycle`).

### Phase 2: Bit-Parallel Reachability Validation (BP-TCRV)
The tape must not contain dead code or unreachable execution paths. The compiler uses a bit-parallel Roy-Warshall algorithm to branchlessly compute the transitive closure of the graph's successor edges:
1. It initializes a 64x64 reachability matrix.
2. It propagates paths using a fully deterministic, fixed 64-iteration loop, utilizing branchless bit-shifts and masking to accumulate reachable routes.
3. It validates that the union of all paths reachable from the `entry_mask` covers all active non-`LoopRedo` nodes on the tape. 

By bounding the matrix to 64x64, the validation takes exactly $O(V^3 / 64)$ steps, remaining deterministic and allocation-free.

## 4. v2 Tape Enhancements

The v2 compiler adds additional mechanisms for the deterministic runtime:
- **LabelSlab**: String labels are interned into a fixed-size `[u8; 1024]` slab array to keep pointers out of the tape struct.
- **ConcurrencyGuardTable**: To restrict concurrent execution, concurrency restrictions are mapped from source-level IDs directly into a compiled table of minimal non-faces. The entries map directly to tape slot indices. During runtime, the hot path evaluates bitmasks against this table using branchless bitset comparisons to guarantee safe concurrency without allocating conflict-resolution state.
