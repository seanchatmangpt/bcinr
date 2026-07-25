# Rule 14: Wrapping vs. Saturation in `bcinr`

According to `AGENTS.md` (Rule 14: Numeric-law requirements), all authoritative arithmetic within the BCINR Deterministic Substrate must be fixed-width, deterministic, and must enforce **saturating or wrapping according to an explicit contract**. 

In the `bcinr` repository, handling numeric boundaries is governed by a strict dichotomy: wrapping is used for **mask generation and bitwise logic**, while saturation is used for **domain clamping and state transitions**. Both methods are utilized to eliminate dynamic control flow hazards and meet the `$CC=1$` (Cyclomatic Complexity = 1) rule.

---

## 1. Wrapping Arithmetic (`wrapping_add` / `wrapping_sub`)

### Primary Use Case: Branchless Mask Generation
Under Rule 9 (Mask-based execution law), runtime predicates must be transformed into full-width masks (`0` or `2^w - 1`). Wrapping subtraction is the standard primitive used to generate these masks mathematically, avoiding `if/else` branching.

### Mechanism
By casting a boolean condition to a `u64` (resulting in `0` or `1`) and subtracting it from `0` via `wrapping_sub`, you generate a full-width bitmask:
* `0u64.wrapping_sub(0)` ➔ `0x00000000_00000000` (All zeros)
* `0u64.wrapping_sub(1)` ➔ `0xFFFFFFFF_FFFFFFFF` (All ones)

### Code Examples in `bcinr`
* **`branchless_priority_queue_pop.rs`**:
  ```rust
  let mask = 0u64.wrapping_sub((val > aux) as u64);
  (val & !mask) | (aux & mask)
  ```
  Here, `wrapping_sub` safely overflows to create a `0xFFFF...` mask if `val > aux`, which is then used to branchlessly select the correct value.

* **`base64_decode_chunk4.rs`**:
  ```rust
  let gt = |a: u64, b: u64| 0u64.wrapping_sub(b.wrapping_sub(a) >> 63);
  ```
  Uses sequential `wrapping_sub` operations to calculate relative distances between characters, extracting the sign bit to generate a comparison mask without branching.

---

## 2. Saturation and Clamping (`saturating_add`, `min`/`max`)

### Primary Use Case: Bounded State Evolution
For mathematical calculations representing real-world metrics, weights, or physical capacities, arithmetic must never overflow (wrap around to 0), which would violate physical conservation laws. Instead, values must "saturate" at their theoretical minimums or maximums.

### Mechanism
Saturation arithmetic enforces hard boundaries on state modifications. Rather than causing panic paths (which violate Rule 3) or silent modulo looping (which violates monotonic mathematics), operations lock at the extremum.

### Code Examples in `bcinr`
* **`add_sat_i32.rs`**:
  ```rust
  ((val as i32).saturating_add(aux as i32)) as u32 as u64
  ```
  Provides a guaranteed fixed-width saturating sum. If the addition exceeds `i32::MAX`, it halts precisely at `i32::MAX`, fulfilling the error envelope rules.

* **`clamp_i64.rs`**:
  ```rust
  let lo = a.min(b);
  let hi = a.max(b);
  v.max(lo).min(hi) as u64
  ```
  Clamps values within a dynamic range using cascaded `.max()` and `.min()` operations.

* **`examples/mask_primitives.rs`** (Branchless Clamp Composition):
  ```rust
  // min(max(val, lo), hi) without any conditional jump
  let clamped = min_u32(max_u32(val, lo), hi);
  ```
  Shows how branchless clamp limits can be composed entirely out of basic mask-based primitives, keeping the entire pipeline allocation-free and branchless.

---

## Summary of the Contract
* **Wrap** when projecting logic into bitmasks, memory indices, or cryptographic primitives.
* **Saturate** when calculating physical states, distances, limits, or iterative mathematical updates.
