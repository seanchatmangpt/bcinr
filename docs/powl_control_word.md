# POWL Control Word (`ctrl`) Encoding

The `ctrl` field inside the `Powl64Op` structure (and its array counterpart in `PowlTapeLarge`) is a 64-bit control word responsible for encoding structural execution flags—most notably the **concurrency marker**—without introducing control-flow branches, adhering strictly to the `bcinr` $CC=1$ mandate.

## Cache-Line Aligned Structure

In `bcinr-powl`, the execution tape is composed of `Powl64Op` structs. To avoid false sharing in concurrent environments, each operation is padded to exactly 64 bytes to fill a CPU cache line. 

```rust
#[repr(C, align(64))]
pub struct Powl64Op {
    pub pred_mask: u64,
    pub succ_mask: u64,
    pub ctrl: u64,          // Control word (u64::MAX signals concurrency marker)
    pub op_kind: OpKind,
    pub choice_group: u8,
    pub depth: u8,
    pub fan_out: u8,
    pub _pad: [u8; 36],     // 64-byte cache-line alignment
}
```

## Encoding Concurrency Markers

The POWL v2 compiler emits a specific sentinel value to signal a parallel/concurrency fan-out gateway: `u64::MAX` (`0xFFFF_FFFF_FFFF_FFFF`).

When an operation acts as a concurrency gateway (usually coupled with `op_kind == OpKind::Concur`), its `ctrl` word is set to `u64::MAX`. Otherwise, it remains `0`. The dispatcher uses this marker to identify points where the workflow must branch out into concurrent execution tracks (triggering the `fanout_pair` mechanism in the 8-lane CAS `BpadDispatcher`).

## Branchless Evaluation ($CC=1$)

Under the strict deterministic laws of `bcinr` (no `if`, `match`, or data-dependent loops), the runtime cannot simply use a conditional jump like `if op.ctrl == u64::MAX { ... }`. 

Instead, the `ctrl` field is evaluated branchlessly using an algebraic bit-twiddling primitive called `eq_mask_u64`. This function returns a full-width mask (`u64::MAX`) if the values are equal, and `0` otherwise.

```rust
#[inline(always)]
pub fn is_concur(&self) -> bool {
    // Evaluates to a boolean without branching
    eq_mask_u64(self.ctrl, u64::MAX) != 0
}

#[inline(always)]
pub const fn eq_mask_u64(a: u64, b: u64) -> u64 {
    let diff = a ^ b;
    // Standard two's-complement zero-detection:
    // (diff | diff.wrapping_neg()) >> 63 equals 1 iff diff != 0, else 0
    let nonzero_bit = (diff | diff.wrapping_neg()) >> 63;
    
    // Maps 0 -> u64::MAX, 1 -> 0
    nonzero_bit.wrapping_sub(1)
}
```

### How `eq_mask_u64` works:
1. **`diff = a ^ b`**: XORs the `ctrl` value with `u64::MAX`. If they are equal, `diff` is exactly `0`.
2. **`nonzero_bit` extraction**: OR-ing `diff` with its two's complement negation sets the most significant bit (MSB) to `1` for any non-zero value. Shifting right by 63 isolates this bit. It yields `1` if the numbers differ, and `0` if they are equal.
3. **`wrapping_sub(1)`**: Subtracting `1` from `0` triggers an arithmetic underflow, wrapping around to `0xFFFF_FFFF_FFFF_FFFF` (`u64::MAX`). Subtracting `1` from `1` yields `0`. 

This resultant mask can be used to branchlessly steer state transitions—via bitwise `&` and `|` (`select` operations)—dictating whether the scheduler should advance a single pipeline or trigger a concurrent fan-out, perfectly preserving constant-time, branchless execution.
