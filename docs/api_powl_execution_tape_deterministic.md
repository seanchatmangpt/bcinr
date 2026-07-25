Here is the documentation on how the Powl Execution Tape maintains zero heap allocation and handles deterministic bounds, based on the `crates/bcinr-powl/src/compiler.rs` analysis.

### Zero Heap Allocation
The `PowlTape` architecture entirely avoids the heap by utilizing flat, pre-sized primitives:
- **Flat Array Storage:** The compiler maps the entire workflow AST into a fixed-capacity, pre-allocated array of 64 slots (`[Powl64Op; 64]`). Allocation simply increments an internal length counter `len` without triggering `malloc`.
- **Bitmask Wiring:** Node relationships (edges) are not stored as pointer graphs or dynamically sized adjacency lists (`Vec`). Instead, they are wired using 64-bit integer bitmasks (`pred_mask` and `succ_mask`), where each bit represents a node index. 
- **Label Interning (v2):** In the v2 compiler, string labels aren't stored as heap-allocated `String`s. Instead, they are packed into a `LabelSlab`, which is a fixed-size `[u8; 1024]` array, and nodes just store a `u16` byte offset pointing into this slab.

### Deterministic Bounds
The compiler uses strict, constant bounds to ensure verifiable and branchless timing-invariant execution:
- **Strict Capacity Cutoff:** The tape enforces a hard $V \le 64$ limit. If an AST evaluates to more than 64 nodes, it deterministically refuses it with a `CompileError::TapeFull` (or `CompileErrorV2::TapeFull`), preventing unbounded growth.
- **Kahn's Validation (Phase 1):** Kahn's topological sort bounds its queue and structures to exactly `[u32; 64]`. It deterministically limits its validation against the allocated node count.
- **Branchless Roy-Warshall Validation (Phase 2):** To validate node reachability (BP-TCRV), the runtime operates on a fixed $64 \times 64$ matrix. It deterministically runs the exact same number of iterations—64 outer loops and 64 inner loops—regardless of the actual node count (padding unused slots with bounds-masking). 
- **Data-Independent Transitions ($CC=1$):** Inside the validation matrices, the math avoids all data-dependent branches (`if`/`else`). Mask manipulation is accomplished using wrapping arithmetic (e.g., `let mask = 0u64.wrapping_sub(can_reach_k)`), ensuring consistent processing time (constant time complexity: $O(V^3 / 64)$ where $V = 64$) and thereby preventing timing side-channel leaks.
- **Re-keyed Concurrency:** Concurrency guards and minimal nonfaces restrict state tracking purely to the `0..64` valid tape slot bounds rather than unbounded arbitrary string maps.
