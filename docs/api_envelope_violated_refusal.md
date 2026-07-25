# Branchless Translation of Envelope Violations to `EnvelopeViolated`

In the BCINR deterministic substrate, the Radon Law ($CC=1$) and Zero-Allocation mandates prohibit panics or control-flow branching (`if`/`else` or `match`) when boundaries (such as a fixed-capacity `BumpArena` out-of-bounds allocation or a mathematical error bound) are violated. Instead, these conditions translate into a bounded `StabilityRefusal::RuntimeEnvelopeViolated` (or `EnvelopeViolated`) typed refusal using strict bitwise polynomials and masked state selection.

## 1. Branchless Evaluation and Mask Generation
When `BumpArena` allocates memory, it computes the new state using wrapping arithmetic rather than early-returning on bounds checks:
```rust
let (next_offset, overflow) = current_offset.overflowing_add(size);
```
The validity condition is evaluated mathematically to produce a $0$ (failure) or $1$ (success), which is then converted into a full-width bitmask without branches:
```rust
let can_alloc = ((next_offset <= self.data.len()) & !overflow) as usize;
let mask = 0usize.wrapping_sub(can_alloc);
```
* If `can_alloc == 1`, `mask` becomes `usize::MAX` (all `1`s).
* If `can_alloc == 0`, `mask` becomes `0` (all `0`s).

## 2. Gating State Mutation
To enforce Rule 10 ("No mutation before complete admission"), the persistent state is only conditionally updated via a branchless bitwise `select` operation:
```rust
self.offset = (next_offset & mask) | (current_offset & !mask);
```
If the envelope boundary is violated (`mask == 0`), the new state is zeroed out and `current_offset` is preserved perfectly, bit-for-bit, without a single `if` statement.

## 3. Branchless Fault Accumulation
As execution continues in constant $O(1)$ time without unwinding, failures (like `None` or fault bits) are accumulated into an opaque `NumericFaultSet` or `RefusalSet` using bitwise `OR` unions (e.g. `self.faults.union(e)`). This prevents "first-error-wins" short-circuiting. 

## 4. Substrate Boundary Signaling (`RuntimeEnvelopeViolated`)
At the edge of the substrate boundary (e.g. `allocator.rs` in `bcinr-cmca`), these raw fault bits are mapped back to bounded enum types safely. In `crates/bcinr-cmca/src/allocator.rs`, `StabilityRefusal::RuntimeEnvelopeViolated` is mapped from integer fault codes branchlessly via `StabilityRefusal::from_u32`:

```rust
pub fn from_u32(val: u32) -> Option<Self> {
    let lookup = [
        /* ... */
        Some(Self::RuntimeEnvelopeViolated), // mapped at index 9
        /* ... */
    ];
    let in_bounds = const_lt_u32(val, 21);
    let idx = const_select_u32(in_bounds, val, 21) as usize;
    lookup[idx & 31]
}
```
This safely exposes the typed refusal to the MAPE-K Autonomic Loop to initiate recovery—entirely preserving branchless determinism.
