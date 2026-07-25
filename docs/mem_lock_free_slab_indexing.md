# LockFreeSlab: Branchless Indexing and ABA-Free Generation Mapping

Based on the implementation in `crates/bcinr-logic/src/abstractions/lock_free_slab.rs` and the architectural guidelines in `docs/mem_lock_free_slab_aba.md`, here is how `LockFreeSlab` handles index mapping and generation counters without branching (adhering to the Radon Law, $CC=1$).

## 1. Monotonic Generation Counter via `AtomicU32`

Standard lock-free freelists push freed indices back onto the head pointer, making them vulnerable to ABA races unless physical indices and generation counters are bit-packed into a single atomic value.

`LockFreeSlab` sidesteps this entirely by abandoning in-band deallocation. The `freelist` atomic head does not track physical indices directly; instead, it acts entirely as a **monotonic generation counter**.

```rust
let next = (head.wrapping_add(1)) & can_alloc_mask | head & !can_alloc_mask;
```

Because the `AtomicU32` only increments, it inherently provides a unique generation for every allocation up to $2^{32}-1$. A thread suspended before a Compare-And-Swap (CAS) cannot experience a false positive because the state never moves backward.

## 2. Branchless State Checks (The Radon Law)

In typical logic, checking for exhaustion (`head == 0xFFFFFFFF`) requires an `if` branch. `LockFreeSlab` implements this mathematically via polynomial bit masks:

```rust
let is_empty = (head == 0xFFFFFFFF) as u32;
let can_alloc = (!is_empty) & 1;
let can_alloc_mask = 0u32.wrapping_sub(can_alloc); // Resolves to 0xFFFFFFFF or 0x00000000
```

This mask forces the CPU to evaluate both the success state (`head.wrapping_add(1)`) and the failure state (`head`), then statically select the correct path without any data-dependent jumps. The final returned result is selected using the same mechanism:

```rust
let cas_success = (cas_res.is_ok() && can_alloc != 0) as u32;
// Computes `head` if successful, or `0` if it failed.
result = head & (0u32.wrapping_sub(cas_success)); 
```

## 3. Out-of-Band Physical Index Mapping (`index % N`)

Because `alloc_t1` returns an endlessly increasing generation counter (`head`), mapping this to a physical slot in a fixed array `[T; N]` is decoupled and happens mathematically at the use-site via `index % N`.

> [!NOTE]
> **Why `index % N` is Branchless**
>
> In BCINR, `N` is constrained as a compile-time constant via const generics (`LockFreeSlab<const N: usize>`). Consequently, `index % N` never relies on a runtime division instruction that could branch (e.g., panicking on zero).
> - If `N` is a power of two, the compiler natively lowers `% N` into a branchless bitwise AND: `index & (N - 1)`.
> - If `N` is not a power of two, the compiler optimizes it into a branchless multiplication trick (Lemire's algorithm or magic number division).

By separating the continuously advancing **generation counter** from the **physical index**, `LockFreeSlab` eliminates the need for bit-shifting complex generation logic into a single pointer, safely enforcing the zero-allocation, branchless $CC=1$ state transitions required by the deterministic substrate.
