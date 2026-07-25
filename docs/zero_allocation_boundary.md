# The Zero-Allocation Boundary in BCINR

According to the `GEMINI.md` project mandate and Rule 3 of the `AGENTS.md` constitution, **The Zero-Allocation Boundary** is a non-negotiable architectural law governing the deterministic substrate. It mandates that hot-path execution must perform exactly **zero heap allocations** and be completely `#![no_std]`. This ensures predictable latency, mathematical branchlessness, and physical impossibility of timing side-channels.

## 1. The `#![no_std]` Requirement

As stated in Rule 3 of `AGENTS.md`, the entire authoritative runtime must run under `#![no_std]`.
- **Complete Decoupling:** The substrate is entirely decoupled from the Rust standard library (`std`). It relies only on the `core` library.
- **No OS Dependencies:** By operating without `std`, the runtime cannot implicitly invoke OS-level routines, thread blocking, or system-dependent I/O, ensuring execution is purely computational and portable across embedded systems or WASM.
- **Structural Guarantees:** `#![no_std]` natively enforces the removal of many implicit, unpredictable, or allocating behaviors found in standard libraries, building a foundation for perfectly bounded execution.

## 2. The Prohibition on Heap Allocation

Rule 3 explicitly lists `no alloc` and `zero heap allocation` as absolute runtime laws.
- **Categorical Ban on Dynamic Memory:** Constructs like `Box`, `Vec`, `String`, and standard garbage collection are entirely banned from the hot path. 
- **Eliminating Side-Channels:** Heap allocation inherently involves non-deterministic delays—such as searching for free blocks, handling fragmentation, and OS-level mutex locking. Banning dynamic allocation makes execution latency completely deterministic and eliminates micro-architectural timing side-channels.
- **Preserving $CC=1$ (The Radon Law):** Allocators require internal control-flow branches (e.g., checking if memory is available, looping through free lists). Relying on an allocator would violate the requirement that the authoritative call graph contains no data-dependent branches or loops.
- **Elimination of Panic Paths:** Out-of-Memory (OOM) situations lead to unpredictable panics. By removing dynamic allocations, memory exhaustion panics are physically impossible, replaced structurally by deterministic, fixed-boundary capacity limits.

## 3. Lawful Memory Management: BumpArena, LockFreeSlab, and Stack Values

Since the authoritative runtime cannot allocate dynamically, it must rely on statically bounded memory access mechanisms allocated exactly once at initialization.

### Fixed-Size Stack Values
Because heap cloning is banned, Rule 10 of `AGENTS.md` requires that memory mutation and state transitions use bounded structures. State mutation is handled via "structural cloning"—copying candidate state into a **fixed-size stack value** or scratch structure, deriving an admission mask, and executing a field-wise masked commit. The execution footprint remains bounded strictly to stack capacity without triggering a heap allocation.

### `BumpArena`
For deterministic $O(1)$ memory allocation, the substrate uses a `BumpArena`. 
- **Branchless Operation:** Instead of conditional `if`/`else` capacity checks, `BumpArena` computes allocations using branchless bitwise masking. 
- **Mechanism:** It increments an internal offset pointer. If the increment exceeds capacity, a masking operation yields a `0` success state, isolating the pointer and preventing out-of-bounds mutation in a strictly $CC=1$ compliant manner.

### `LockFreeSlab`
For managing fixed-size memory slots (like concurrent state tracking), the substrate relies on a `LockFreeSlab`.
- **$O(1)$ Atomic Freelist:** It manages fixed capacity slots via an atomic freelist.
- **Loop-Free Operations:** Traditional concurrent lock-free data structures use Compare-and-Swap (CAS) `while` loops. However, loops violate Rule 3 (`no data-dependent loop termination`). Therefore, `LockFreeSlab` strictly avoids looping by using single, bounded atomic transitions and branchless bitwise derivations to safely yield available memory slots in constant time.
