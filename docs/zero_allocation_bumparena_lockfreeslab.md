# Zero-Allocation Boundary: `BumpArena` and `LockFreeSlab`

In the `bcinr` deterministic substrate, the hot path operates under the strict **Zero-Allocation Boundary** (as mandated by `GEMINI.md` and `AGENTS.md` Rule 3). This requires the execution to be `#![no_std]`, perform zero heap allocations, have fixed bounded memory access, and strictly maintain $CC=1$ (the Radon Law, forbidding any data-dependent branches or loops).

To manage memory within these rigid constraints, the system utilizes two core abstractions: `BumpArena` and `LockFreeSlab`. These tools manage pre-allocated regions of memory (often statically sized buffers) using branchless arithmetic, completely circumventing the system allocator.

## 1. `BumpArena`

The `BumpArena` (`crates/bcinr-logic/src/abstractions/bump_arena.rs`) provides a branchless bump allocator for deterministic, $O(1)$ memory allocation without heap fragmentation. It operates by maintaining an `offset` and a `capacity`. 

To allocate memory without branching or panicking on out-of-memory conditions, it uses **bitwise mask selection** (Rule 9: Mask-based execution law).

### Branchless Operation
Instead of using an `if next_offset <= capacity` branch, the allocator evaluates the bounds condition into a `1` or `0`, and then expands it into a full-width bitmask (`0xFFFFFFFF` or `0x00000000`).

```rust
let next_offset = current_offset.wrapping_add(size);
let success = (next_offset <= self.capacity) as u32;
let mask = 0u32.wrapping_sub(success);

// Mask-based state transition
self.offset = (next_offset & mask) | (current_offset & !mask);
```

If the allocation fits, `mask` is all ones, and `self.offset` is updated to `next_offset`. If it exceeds capacity, `mask` is all zeros, and `self.offset` safely falls back to `current_offset`. The allocator returns the result and the mask, allowing the caller to branchlessly handle the success or failure of the allocation.

## 2. `LockFreeSlab`

The `LockFreeSlab` (`crates/bcinr-logic/src/abstractions/lock_free_slab.rs`) provides deterministic $O(1)$ allocation and deallocation for uniformly sized items. It relies on `AtomicU32` and a fixed-size `const N: usize` array (`next_indices`), satisfying the fixed bounded memory requirement.

### Branchless Atomic Operations
Like the `BumpArena`, it strictly avoids branches and loop backedges. A traditional lock-free freelist might loop until a Compare-and-Swap (CAS) succeeds. Under the `bcinr` laws (Rule 13: No unbounded execution), a retry loop would be a violation. Instead, `LockFreeSlab` performs a bounded operation completely devoid of branching.

```rust
let is_empty = (head == 0xFFFFFFFF) as u32;
let can_alloc = (!is_empty) & 1;
let can_alloc_mask = 0u32.wrapping_sub(can_alloc);

// Determine the next head purely through arithmetic masks
let next = (head.wrapping_add(1)) & can_alloc_mask | head & !can_alloc_mask;

// Single Compare-and-Swap attempt
let cas_res = self.freelist.compare_exchange_weak(
    head,
    next,
    Ordering::Relaxed,
    Ordering::Relaxed,
);
```

The success of the atomic operation is evaluated without any conditionals:
```rust
let cas_success = (cas_res.is_ok() && can_alloc != 0) as u32;
```

### Synthesis
Both `BumpArena` and `LockFreeSlab` allow the `bcinr` runtime to manage dynamic state requirements within a strictly `#![no_std]` environment. By pre-allocating the required bounds and applying bitwise polynomials and atomic masks, the substrate completely bypasses heap allocation and achieves branchless ($CC=1$) memory mutations.
