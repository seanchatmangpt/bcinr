# POWL VM: Branchless and Allocation-Free State Management

The POWL (Partially Ordered Workflow Language) VM in `bcinr-powl` handles state tracking and memory management strictly without heap allocations and without control-flow branches. This ensures zero timing side-channels and meets strict deterministic execution requirements. 

The architecture divides state into three primary components:

## 1. Zero-Allocation Memory Management (`tape.rs`)
Memory management is strictly stack-based and fixed-size. The VM avoids heap allocations by embedding the entire program structure into inline arrays:
- **`Powl64Op`**: A single operation packed into a fixed structure (a 64-byte cache line in `v2`, or a 32-byte struct in legacy). It encodes dependencies and branches via `u64` bitmasks instead of pointers or variable-length collections.
- **`PowlTape` & `PowlTapeLarge`**: `PowlTape` accommodates up to 64 operations in a flat `[Powl64Op; 64]` array. For larger workflows, `PowlTapeLarge` scales to 512 ops using arrays of bitmasks (`[[u64; 8]; 512]`). 
- **`LabelSlab`**: String labels are interned into a fixed `[u8; 1024]` stack-allocated byte array using linear scanning, preventing the need for dynamic `String` allocations.

## 2. Branchless Run State (`scheduler.rs`)
Runtime execution state is maintained in `PowlRunState`, which tracks up to 64 operations concurrently using densely packed 64-bit masks:

```rust
#[derive(Clone)]
#[repr(C, align(8))]
pub struct PowlRunState {
    pub done_mask: u64,
    pub active_mask: u64,
    pub check_mask: u64,
    pub choice_taken: u64,
    pub loop_iters: [u8; 64],
    pub tick: u32,
    _pad: [u8; 4],
}
```
The scheduler’s hot-path (`scheduler_tick`) advances execution using **SWAR (SIMD-within-a-register)** techniques rather than branching:
- **Predication**: Readiness checks are calculated bitwise: `(required & !done) == 0`. Instead of branching on the boolean result, it computes a full-width mask via two's complement: `0u64.wrapping_sub(is_satisfied)`.
- **Applying Effects Unconditionally**: Dispatch logic (e.g., `apply_xor_dispatch`, `apply_loop_redo`) evaluates for every candidate. The side effects are bitwise ANDed with an `active` mask. If the branch shouldn't trigger, the mask is `0`, causing a bitwise no-op on the state registers.
- **Loops & Saturation**: Counters (like `loop_iters`) are updated branchlessly using `saturating_add((active & 1) as u8)`.

## 3. Typestate & Defect Tracking (`typestate.rs`)
Execution safety and correct topology dispatch is enforced by `ExecutionToken`. 

To maintain constant-time execution without throwing exceptions or branching during active evaluation, the token uses branchless accumulation for error tracking:
- Defect registers (`defect_double_fire`, `defect_invalid`, `defect_malformed`) begin at `0`.
- When an operation is marked as fired (`consume_op`), invalid bits, out-of-bounds accesses, and double-fires are identified using bitwise XORs and ANDs.
- These defects are accumulated in the status registers without short-circuiting. The execution is only refused at the end of the transaction (`assert_exhausted`), ensuring the hot path is free from data-dependent conditional bounds checks.
