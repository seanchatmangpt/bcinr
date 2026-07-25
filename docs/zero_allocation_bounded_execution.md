# Zero Allocation and Fixed Bounded Execution in BCINR

According to **Rule 3 (Absolute runtime laws)** of the `AGENTS.md` constitution, the authoritative runtime in BCINR must preserve a rigid pipeline: 
$\text{admitted input} \rightarrow \text{fixed instruction shape} \rightarrow \text{deterministic output}$. 

Two primary pillars of this pipeline are **zero heap allocation** and **fixed bounded execution work**. Here is an analysis of how these principles operate, why abstractions cannot cheat them, and how they relate to the project's foundational data structures.

## 1. The Core Requirements

- **Zero Heap Allocation**: No authoritative hot-path code may dynamically allocate memory on the heap. Rust's `alloc` crate, global allocator symbols, `Vec`, `Box`, and `String` are entirely banned. 
- **Fixed Bounded Execution Work**: The runtime must not include `while` loops, data-dependent `for` loops, variable recursion, or iterator short-circuiting. Iteration must be compile-time fixed, macro-unrolled, or mathematically proven to contain no variable loop backedges in the final release object code.

## 2. The Fallacy of the Fixed-Size API Backed by Heap Allocation

Rule 3 explicitly states: *"A fixed-size API backed by a heap allocation is a violation."* 

Even if an API appears structurally fixed to the caller (e.g., exposing a bounded interface or returning a fixed-size struct), using heap allocation internally is unlawful because it introduces **hidden variability**:

1. **Unbounded Allocator Latency**: Global allocators are incredibly complex state machines. They acquire locks, search for free memory blocks (causing unbounded loops), manage fragmentation, and potentially trigger OS system calls (like `mmap` or `sbrk`) resulting in context switches and page faults. This destroys determinism.
2. **Hidden Panic Paths**: Heap allocation can fail (Out of Memory). In standard Rust, an OOM triggers an uncatchable panic. BCINR strictly outlaws panic paths; failures must be represented via bounded, typed refusals (e.g., `Result::Err`). 
3. **Cache Invalidation & Indirection**: Heap-allocated pointers inherently fragment spatial locality. Predicting memory access latency becomes impossible when physical layout varies wildly based on global allocator history.

An API is not judged by its signature alone; its *transitive call graph* and *underlying mechanics* must obey the constitution.

## 3. Enforcing Physical Execution Determinism in the Hot Path

BCINR is mandated to serve as a **"hard substrate for AGI,"** where timing side-channels and latency jitter are physically impossible. The rules combine to guarantee cycle-exact determinism:

* **Cyclomatic Complexity = 1 (CC=1)**: By transforming sequential control flow into bitwise masks, SWAR (SIMD Within A Register), and arithmetic selection, the CPU branch predictor is effectively bypassed. 
* **Zero Allocation**: Removes the OS and global allocator from the hot path entirely.
* **Bounded Work**: Guarantees that every execution path executes the exact same number of instructions.

Together, these rules ensure that an execution trace in the hot path is physically identical for all inputs. The runtime operates like a pure boolean circuit—the computation is always exactly $O(1)$ relative to the fixed domain size. It cannot be "faster" or "slower" based on the data. 

## 4. Bounded Memory Mechanics: `BumpArena` and `LockFreeSlab`

Since the heap is forbidden, BCINR requires specialized, allocation-free memory management structures to handle autonomic state transitions (the MAPE-K loop) and temporal scratch space. As noted in `GEMINI.md`, memory is strictly managed via `BumpArena` and `LockFreeSlab`.

These structures adhere to Rule 3 by strictly pre-allocating memory during node initialization (outside the hot path) and managing it deterministically at runtime:

* **`BumpArena`**: Used for deterministic scratch space and temporal memory. A bump allocator simply increments a pointer across a pre-reserved, static block of memory. 
  * **Determinism**: Allocation is a single $O(1)$ atomic integer addition. There is no search for free space, no fragmentation tracking, and no OS interaction.
  * **Bounded Work**: Resetting the arena resets a single pointer back to zero, taking $O(1)$ bounded work regardless of how many objects were allocated.
* **`LockFreeSlab`**: Used for dynamic-like lifetimes (e.g., object pooling and autonomic state nodes) without the heap. 
  * **Determinism**: Rather than scanning an array to find an empty slot (which would violate fixed loop bounds), a slab uses fixed-width bit-masks or a lock-free $O(1)$ freelist (like a stack of available indices) to pop an available index.
  * **Safety**: Provides lock-free concurrency bounds without blocking, allowing isolated access in constant time.

Both structures decouple the **logical lifetime** of memory from the **physical allocation** of memory, ensuring the authoritative pipeline can observe, infer, propose, and execute purely mathematically without violating the substrate's absolute laws.
