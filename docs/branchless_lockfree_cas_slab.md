# Branchless Lock-Free CAS Technique in `LockFreeSlab`

The `LockFreeSlab` allocator in `bcinr` achieves a strict $O(1)$, loop-free allocation by eliminating the conventional `while let Err` CAS retry loop. This adheres to the **Radon Law** ($CC=1$) and guarantees bounded execution time without data-dependent control flow.

## The Mechanism

The single-pass allocation is implemented in the `alloc_t1` function. It attempts to acquire an index from an `AtomicU32` freelist using exactly one Compare-And-Swap (CAS) operation, yielding a deterministic boolean success flag rather than retrying on contention.

### 1. Branchless Mask Generation
First, the current state of the freelist is loaded:
```rust
let head = self.freelist.load(Ordering::Relaxed);
```
Rather than branching on whether the freelist is empty (`0xFFFFFFFF`), the logic computes a full-width bitmask (`can_alloc_mask`):
```rust
let is_empty = (head == 0xFFFFFFFF) as u32;
let can_alloc = (!is_empty) & 1;
let can_alloc_mask = 0u32.wrapping_sub(can_alloc); // 0xFFFFFFFF if valid, 0x00000000 if empty
```

### 2. Constant-Time State Selection
The next state is calculated using bitwise selection, unconditionally computing both paths but selecting the correct one based on the mask:
```rust
let next = (head.wrapping_add(1)) & can_alloc_mask | head & !can_alloc_mask;
```

### 3. Single-Shot CAS
Instead of a retry loop, it fires a single `compare_exchange_weak`:
```rust
let cas_res = self.freelist.compare_exchange_weak(
    head,
    next,
    Ordering::Relaxed,
    Ordering::Relaxed,
);
```

### 4. Branchless Result Processing
The outcome of the CAS is converted back into deterministic primitive integers:
```rust
let cas_success = (cas_res.is_ok() && can_alloc != 0) as u32;
success = cas_success;
result = head & (0u32.wrapping_sub(cas_success));
```
- If the CAS succeeds and an allocation was valid, `cas_success` is `1`. `result` becomes `head` (the allocated index).
- If the CAS fails (due to contention, spurious failure, or empty freelist), `cas_success` is `0`. `result` becomes `0`.

## Why this approach?

By abandoning the typical `while` loop, this technique guarantees a fixed execution budget (e.g., ~5 ns for the primitive, bounded to ≤ 200 ns aggregate budget) and enforces constant cyclomatic complexity ($CC=1$). Any allocation failure (from contention) is simply returned to the caller, pushing retry logic out of the hot-path allocation primitive and into the high-level autonomic MAPE-K loop, ensuring deterministic execution time for the substrate.
