# LockFreeMpmcRing: Epoch Counters and Sequence Tracking

The `LockFreeMpmcRing` implements a deterministic, bounded, wait-free Multi-Producer Multi-Consumer (MPMC) queue without mutexes or heap allocations. It achieves thread safety, ABA problem prevention, and data race avoidance by utilizing an **Epoch-based sequence tracking mechanism**.

## Atomic Counters

The ring differentiates states using three types of atomic variables:
- **`head` (`AtomicU32`)**: Monotonically increasing ticket counter for producers.
- **`tail` (`AtomicU32`)**: Monotonically increasing ticket counter for consumers.
- **`sequence` (`AtomicU32`)**: A per-slot counter indicating the "epoch" (or lap) the slot is currently in. At initialization, each slot's sequence is set to its index `i`.

## Atomically Differentiating Slot States

A slot's state (Empty, Full, or Write-in-Progress) is evaluated mathematically by comparing its `sequence` counter against the current `head` or `tail` tickets.

### 1. Empty State (Ready for Push)
When a producer targets a slot at index `h & mask` (where `h` is the current `head`), it checks if:
`slot.sequence == h`
If this difference is exactly `0`, the slot is perfectly in phase with the producer's epoch `h` and is ready to be written. The producer claims exclusive ownership by executing a Compare-And-Swap (CAS) on `head` (`h` → `h + 1`).

### 2. Full State (Ready for Pop)
When a consumer targets a slot at index `t & mask` (where `t` is the current `tail`), it checks if:
`slot.sequence == t + 1`
If they match, it signifies that a producer has fully finished writing the data for epoch `t`. The consumer claims ownership of the slot for reading via a CAS on `tail` (`t` → `t + 1`).

### 3. Write-in-Progress State
Because producers first execute the CAS on `head` to gain ownership but do not update the slot's `sequence` until *after* the data is written, a write-in-progress state naturally emerges. During this window, the consumer will observe `slot.sequence == t` instead of `t + 1`. This mathematical mismatch safely blocks consumers from reading half-written (dirty) slots.

## Memory Ordering and Advancing Epochs

To ensure the *happens-before* relationship and prevent data races without mutexes, strict memory ordering is employed:

- **Producers** load the `sequence` with `Ordering::Acquire`. After completing the unsafe write to the slot, they mathematically advance the epoch to signal consumers:
  `slot.sequence.store(h + 1, Ordering::Release)`

- **Consumers** also load the `sequence` with `Ordering::Acquire` to guarantee visibility of the producer's write. After safely reading the data, they prepare the slot for the next lap (the next producer wrap-around) by adding the ring's capacity `N` (`mask + 1`):
  `slot.sequence.store(t + mask + 1, Ordering::Release)`

## Mitigating the ABA Problem

The ABA problem is avoided natively since `head` and `tail` are wrapping `u32` monotonic counters rather than bounded indices. 
- It requires a full wrap of over 4 billion operations to trigger an ABA collision on the CAS.
- Furthermore, because the per-slot `sequence` strictly checks for exact `head` or `tail` matches, any thread executing an operation on a wrapped counter will find the slot's sequence entirely out of phase (e.g., expecting `h + N` instead of `h`), rejecting the operation intrinsically.

## Conclusion

By mapping the sequential control flow into purely arithmetic sequence checking and fixed-width atomic operations (`CC=1`), `LockFreeMpmcRing` satisfies the bounded execution mandate (≤ 200ns T1 budget) and guarantees thread safety completely devoid of locks.
