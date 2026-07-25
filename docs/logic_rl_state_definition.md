I have found the `RlState` struct in the codebase and analyzed its implementation.

File: `/Users/sac/bcinr/crates/bcinr-logic/src/autonomic/rl_state.rs`

### Exact Struct Fields

The `RlState` struct is composed of exactly 136 bits and uses a `#[repr(C)]` layout for structural integrity:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(C)]
pub struct RlState {
    /// bits 0-63
    pub low: u64,
    /// bits 64-127
    pub high: u64,
    /// bits 128-135
    pub extra: u8,
}
```

### Compliance with Zero-Allocation Boundary

1. **Stack Allocation**: The struct relies purely on fixed-width primitives (`u64` and `u8`), maintaining a tight, fixed size of 136 bits. It completely eliminates heap allocation in alignment with the "dteam spec" to prevent heap churn.
2. **Deriving `Copy` and `Clone`**: `RlState` derives traits like `Copy`, which enforces that any state cloning occurs as a cheap stack copy rather than requiring heap-backed allocation.
3. **No Dynamic Data**: It contains no references (`&`, `Box`, `String`, or `Vec`), ensuring that initialization and propagation are entirely done on the stack.
4. **Branchless Merging (CC=1)**: Operations that merge state (like `RlState::merge`) exclusively use deterministic bitwise XOR operations (`^`) without resorting to `if`/`else` control flow or dynamic allocations, ensuring consistent constant-time transitions.
