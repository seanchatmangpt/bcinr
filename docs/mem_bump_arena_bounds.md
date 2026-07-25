# Branchless Memory Allocation and Typed Refusals in BCINR

Under the strict mandates of the Radon Law ($CC=1$) and the Zero-Allocation Boundary in the BCINR deterministic substrate, memory allocation cannot use branching control flow (`if/else` or `match`) or trigger a panic when fixed capacities are exceeded. Instead, it must rely on branchless arithmetic and produce mathematical typed refusals like `StabilityRefusal::EnvelopeViolated`. 

Here is how the $O(1)$ memory allocator (`BumpArena` in `crates/bcinr-logic/src/mem.rs` and `crates/bcinr-logic/src/abstractions/bump_arena.rs`) structurally enforces this.

## The Constant-Time `alloc` Mechanics

The `BumpArena` refuses allocations that exceed fixed byte capacity through a combination of safe wrapping arithmetic and a bitwise selection mask.

### 1. Wrapping Arithmetic
Instead of bounds-checking prior to addition (which often compiles down to a conditional jump), the allocator calculates the tentative next offset using wrapping arithmetic. 
```rust
let current_offset = self.offset;
let (next_offset, overflow) = current_offset.overflowing_add(size);
```

### 2. Branchless Success Evaluation
The success of the allocation is evaluated as a mathematical integer rather than a boolean controlling a branch. It checks that the offset remains within the capacity bounds and that no overflow occurred:
```rust
let can_alloc = ((next_offset <= self.data.len()) & !overflow) as usize;
```

### 3. Mask Generation
The `can_alloc` result (which is `1` for success and `0` for failure) is mathematically transformed into a full-width bitwise mask:
```rust
let mask = 0usize.wrapping_sub(can_alloc);
```
- If `can_alloc == 1`, `0 - 1 = usize::MAX` (all `1`s).
- If `can_alloc == 0`, `0 - 0 = 0` (all `0`s).

### 4. Masked State Selection
The arena's state is mutated via a branchless bitwise polynomial. It selects the `next_offset` if the allocation was successful, and reverts to the `current_offset` if the byte capacity was exceeded:
```rust
self.offset = (next_offset & mask) | (current_offset & !mask);
```
If the allocation boundary is violated, the `mask` is zero. The `next_offset` is zeroed out, and the `current_offset` is perfectly preserved. The state remains bit-for-bit unchanged without ever executing an `if` condition.

### 5. Translating to `EnvelopeViolated` Refusals
Because the core logic is strictly numeric and total over its domain, it never panics or early-returns. Instead:
- In `try_alloc`, it directly returns `(current_offset & mask, mask)`.
- In `alloc`, it safely returns an `Option<&mut [u8]>` using the `(can_alloc != 0).then(...)` mapping. 

The resulting failure mask or `None` propagates to the substrate boundary (e.g., via mappers like `wrap_result` in `allocator.rs`), where the failure is unwrapped and securely translated into a bounded typed refusal such as `StabilityRefusal::RuntimeEnvelopeViolated`. This cleanly signals the MAPE-K Autonomic Loop to initiate recovery, ensuring perfect branchless determinism.
