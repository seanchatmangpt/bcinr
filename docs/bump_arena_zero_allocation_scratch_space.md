# `BumpArena` and Bounded Scratch Space in BCINR

In the BCINR deterministic computational substrate, **Rule 3** mandates an absolute strictness of `no alloc` and `zero heap allocation`. The system must operate within a fixed memory envelope (`#![no_std]`). To satisfy this rule while supporting complex, variable-sized operations during the MAPE-K loop, BCINR relies on the `BumpArena` abstraction. 

## 1. Zero-Allocation Boundary and the `BumpArena`
The `BumpArena` is a branchless bump allocator that manages a contiguous, pre-allocated, fixed-size memory span. By doing so, it fulfills Rule 3 by completely avoiding the heap. It does not issue dynamic memory allocation requests (`malloc` or `Box`), nor does it suffer from heap fragmentation. 

## 2. Scratch Space for the Speculative Candidate State
During the **Propose** phase of the MAPE-K autonomic loop, the system generates a **Speculative Candidate State**. This is a transient, intermediate calculation of a potential state transition. 

- **Stack Allocation:** For fixed-size, strictly bounded computations (like transition arrays in a `petri_tick`), the candidate state is calculated purely on the stack. The hardware stack pointer inherently handles memory deterministic reclamation.
- **`BumpArena` Allocation:** When variable-sized scratch space is required across multiple operations or phases to assemble complex speculative state, the `BumpArena` is utilized. It acts as a deterministic $O(1)$ scratch pad. The system "bumps" a cursor forward to reserve space for the speculative data without ever touching the heap.

Crucially, because this candidate state is *speculative*, it is strictly decoupled from the persistent state. If the candidate fails the Hoare contract predicates during the subsequent **Accept** phase (resulting in a $0$ admission mask), the persistent state remains untouched, adhering to Rule 10 (No mutation before complete admission).

## 3. Eliminating Allocation Timing Variance (Radon Law $CC=1$)
The core innovation of the `BumpArena` is that it provides this scratch space without introducing **allocation timing variance**. In traditional systems, an arena uses conditional bounds-checking (`if size + offset <= capacity`), which introduces branching and variable execution time.

To satisfy the **Radon Law ($CC=1$)**, `BumpArena` implements bounds-checking via branchless bitwise polynomials:
```rust
let current_offset = self.offset;
let next_offset = current_offset.wrapping_add(size);
let success = (next_offset <= self.capacity) as u32;
let mask = 0u32.wrapping_sub(success); // 0xFFFFFFFF on success, 0x00000000 on fail

// Branchless commit of the new offset
self.offset = (next_offset & mask) | (current_offset & !mask);
```
- **Constant Time:** The calculation executes the exact same mathematical operations regardless of whether there is enough capacity. 
- **Graceful Rejection:** If the arena is full, `success` is $0$, resulting in a `mask` of `0x00000000`. The bitwise selection mathematically drops the requested `size` and retains `current_offset`. The allocation is rejected gracefully without exceptions, panics, or timing leaks.

## 4. Holistic $O(1)$ Reclamation
At the end of an execution epoch (or when triggered by Autonomic Exhaustion metrics), the scratch space must be reclaimed for the next Propose cycle. The `BumpArena` does not use garbage collection or individual deallocation. 

Instead, a branchlessly derived `trigger_mask` dictates an epoch reset:
```rust
self.arena.offset &= !trigger_mask;
```
If the epoch trigger is active (all 1s), `!trigger_mask` evaluates to `0`, instantly zero-clearing the arena's cursor in a single, constant-time arithmetic operation. This reclaims the entire contiguous block of scratch space for the next MAPE-K cycle, ensuring unbounded continuous operation within a hard-bounded memory footprint.
