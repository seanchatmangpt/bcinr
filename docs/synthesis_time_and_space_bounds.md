# Synthesis of Time and Space Bounds in BCINR

In `bcinr`, achieving a mathematically bounded execution footprint—both in latency (time) and memory (space)—is the foundational requirement for creating a "hard substrate" for AGI. The system rejects probabilistic execution and relies on a synthesis of structural constraints to eliminate non-determinism, unpredictable resource consumption, and micro-architectural timing side-channels. 

This bounded footprint is synthesized through the strict intersection of four core pillars: the Radon Law ($CC=1$), Mask-Based Selection, `BumpArena`, and `LockFreeSlab`.

## 1. Bounded Time: The Radon Law ($CC=1$)

The **Radon Law** dictates that every authoritative function must maintain a Cyclomatic Complexity (CC) of exactly 1. 

* **No Data-Dependent Control Flow:** All `if` statements, `match` blocks, and data-dependent loop terminations are strictly prohibited in the hot path.
* **Instruction-Level Determinism:** By eliminating branches, the authoritative call graph generates a fixed sequence of machine instructions. The cycle count remains constant regardless of the semantic payload or input domain.
* **Eradication of Side-Channels:** Because execution time never varies based on data, it is physically impossible for adversaries to infer internal state or cryptographic secrets through timing side-channels. Execution latency is strictly, mathematically bounded.

## 2. Mask-Based Selection: The Mechanism of Constant-Time Logic

To comply with the Radon Law, traditional control-flow logic must be translated into pure arithmetic. This is achieved through **Mask-Based Selection**.

* **Logic as Arithmetic:** Sequential semantic decisions are transformed into bitwise polynomials and full-width masks (e.g., yielding `0` or `!0`).
* **Fixed Execution Work:** Instead of conditionally executing a path (e.g., `if valid { candidate } else { current }`), the substrate computes *both* outcomes and uses a selection function mathematically equivalent to `(mask & candidate) | (!mask & current)`.
* **Safe State Mutations:** Persistent state is never speculatively mutated. The system computes a candidate state into a fixed-size stack value, derives an admission mask, and executes a field-wise masked commit. The execution footprint and computational work are completely invariant to whether the operation is "accepted" or "rejected."

## 3. Bounded Space: The Zero-Allocation Boundary

To complement temporal bounds, `bcinr` enforces strict spatial determinism. The hot path is entirely `#![no_std]` and strictly prohibits heap allocations (e.g., no `Box`, `Vec`, or standard memory allocators). Dynamic memory introduces non-deterministic latency (searching for free blocks, OS locks) and unbounded space risks (fragmentation, Out-Of-Memory panics). 

Instead, memory is managed via statically bounded, pre-allocated structures:

### `BumpArena`: Contiguous, Epoch-Bound Memory
For variable-sized or sequential allocations, `bcinr` uses the `BumpArena`.
* **Deterministic $O(1)$ Allocation:** It advances an internal offset pointer over a fixed-capacity buffer.
* **Branchless Mechanics:** Instead of conditional capacity checks, concurrent allocations use a loop-free atomic `fetch_add` strategy combined with bitwise masking. If a request exceeds capacity, the mask zero-outs the pointer update, rejecting the allocation within strict $CC=1$ constraints.
* **Bounded Lifetime:** Memory is reclaimed holistically at the end of an epoch, guaranteeing that spatial consumption never exceeds the fixed arena size.

### `LockFreeSlab`: Fixed-Size, Independent Lifecycles
For independent, heterogeneous lifecycles (e.g., concurrent state tracking), the substrate relies on the `LockFreeSlab`.
* **Loop-Free Concurrency:** Traditional lock-free data structures rely on unbounded Compare-And-Swap (CAS) `while` loops, which violate bounded execution laws. `LockFreeSlab` manages an atomic freelist using bounded, single-pass, mask-based CAS state transitions without loop backedges.
* **Constant Spatial Footprint:** The slab is bounded to a constant `N` capacity at initialization. It immediately recycles fixed-size slots via its atomic indices, completely eliminating memory exhaustion panics and preventing unbounded growth over time.

## Conclusion

The synthesis of these four mechanisms creates an execution environment that is mathematically constrained in all dimensions:

1. **Space:** Constrained by the Zero-Allocation Boundary via `BumpArena` and `LockFreeSlab`, physically capping memory usage and avoiding dynamic fragmentation.
2. **Time:** Constrained by the Radon Law ($CC=1$) and Mask-Based Selection, converting all logic into fixed-width arithmetic that consumes a constant CPU cycle count.

The result is a deterministic substrate where execution footprint—both spatial and temporal—is an axiomatic guarantee rather than an empirical probability.
