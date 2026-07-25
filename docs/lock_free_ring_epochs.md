# LockFreeMpmcRing: Epoch Tracking and Concurrency Mechanics

The `LockFreeMpmcRing` in the `bcinr` codebase implements a high-performance, lock-free Multi-Producer Multi-Consumer (MPMC) queue. To guarantee thread safety, prevent ABA problems, and avoid data races without relying on mutexes or heap allocations, it utilizes an **Epoch-based sequence tracking mechanism**.

## 1. The Epoch Mechanism (Sequence Counters)

Instead of relying on locking, the ring uses three types of atomic counters to track state and epochs:
- **`head` (AtomicU32)**: Monotonically increasing counter indicating the next ticket for producers.
- **`tail` (AtomicU32)**: Monotonically increasing counter indicating the next ticket for consumers.
- **`sequence` (AtomicU32) per slot**: Each `Slot<T>` contains its own sequence counter that dictates which "epoch" (or lap) the slot is currently in.

At initialization, the `sequence` for each slot is set to its index `i`.

### The Push Epoch (Producer)
1. A producer reads the current `head` (`h`).
2. It targets the slot at index `h & mask` (where `mask` is `Capacity - 1`).
3. It checks if `slot.sequence == h`. If they match, the slot is empty and ready to be written for this exact epoch.
4. The producer claims the slot via a Compare-And-Swap (CAS) on `head` (`h` → `h + 1`).
5. After exclusively writing the data to the slot, the producer advances the slot's epoch by setting `slot.sequence = h + 1`. This signals to the consumer that the slot now contains data for epoch `h + 1`.

### The Pop Epoch (Consumer)
1. A consumer reads the current `tail` (`t`).
2. It targets the slot at index `t & mask`.
3. It checks if `slot.sequence == t + 1`. If they match, the producer has finished writing the data for this epoch.
4. The consumer claims the slot via CAS on `tail` (`t` → `t + 1`).
5. After reading the data, the consumer advances the slot's epoch for the next lap by setting `slot.sequence = t + mask + 1` (effectively `t + N`, where `N` is the capacity). This signals to the producers that the slot is vacant and ready for the next wrap-around.

## 2. Preventing the ABA Problem

The **ABA problem** occurs when a memory location is read twice, has the same value both times, and thus appears unchanged, even though another thread modified it in between.

The ring prevents this natively via **monotonically increasing ticket counters (`head` and `tail`)**:
- Instead of CASing on raw pointers or bounded indices (0 to N-1), `head` and `tail` continually increment as wrapping `u32` values. 
- A full wrap-around of `u32` (over 4 billion operations) would be required to trigger an ABA collision on the CAS.
- Furthermore, the `sequence` counter on each slot strictly matches the exact `head` or `tail` value for that specific lap. A producer arriving at a slot will only attempt to write if the slot's sequence exactly matches its `head` ticket. Even if another thread wraps around the ring, the sequence counter will be out of phase (e.g., expecting `h + N`), inherently preventing ABA conflicts on a specific slot.

## 3. Preventing Data Races and Dirty Slot Reads

Data races and dirty reads are prevented without mutexes through a combination of **CAS Linearization** and strict **Memory Ordering**:

- **Exclusive Ownership via CAS**: Although multiple threads can race for a slot and check the sequence concurrently, only one thread will successfully execute the CAS on `head` or `tail`. The winner gains exclusive rights to access the underlying `UnsafeCell`. Losers safely write to or read from a local dummy variable that is immediately discarded, which completely avoids dangling pointer construction.
- **Acquire/Release Semantics**: 
  - Producers store the updated sequence counter with `Ordering::Release` after writing the data.
  - Consumers load the sequence counter with `Ordering::Acquire` before verifying the epoch.
  - This establishes a strict *happens-before* relationship. The consumer is guaranteed to see the producer's fully completed write in the `UnsafeCell`. It is mathematically impossible for a consumer to read a half-written (dirty) slot because the `sequence` update acts as a memory barrier.

## 4. Zero Heap Allocations and Mutex-Free Design

- **Zero Allocations**: The ring strictly enforces the core project mandate (Zero-Allocation Boundary). The `LockFreeMpmcRing` and all its `Slot<T>` elements are pre-allocated at compile time as a fixed-size contiguous array (`[Slot<T>; N]`). No heap allocations occur dynamically during hot-path execution.
- **Mutex-Free (Wait-Free Bounded)**: Slot ownership is negotiated purely via `AtomicU32` sequence checking and CAS instructions. The data structure avoids OS-level blocking, context switches, or lock contention entirely. Operations guarantee wait-free execution within a bounded maximum of 10 retries to satisfy strict T1 aggregate budgets (≤ 200ns).
