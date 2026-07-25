Here is the requested research on "The Zero-Allocation Boundary" based on `GEMINI.md`:

### The Zero-Allocation Boundary

According to the **Core Architectural Laws** in `GEMINI.md` (`/Users/sac/bcinr/GEMINI.md`), "The Zero-Allocation Boundary" establishes a strict rule for memory management within the hot-path execution of the `bcinr` (BranchlessCInRust) library. 

It explicitly mandates the following:

1. **No Standard Library (`#![no_std]`)**: Hot-path execution must run in a `no_std` environment, entirely decoupled from standard OS-level abstractions and the standard library.
2. **Zero Heap Allocations**: The system must perform exactly **0 heap allocations**. The hot path is prohibited from dynamically requesting heap memory.
3. **Specific Memory Management Structures**: Since dynamic heap allocations are forbidden, memory is mandated to be managed exclusively via the following specialized, allocation-free structures:
   * **`BumpArena`**: Used for bounded, stack-like or contiguous arena allocations where memory is pre-allocated and handed out by simply bumping a pointer.
   * **`LockFreeSlab`**: Used to manage pre-allocated pools of uniformly sized objects without requiring runtime locks, ensuring that execution avoids allocation overhead, blocking, and timing side-channels.
