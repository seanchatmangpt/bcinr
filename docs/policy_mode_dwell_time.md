I have completed the search and analysis of how the Mode Dwell Time requirement is evaluated branchlessly.

Here is the markdown document detailing the findings from the source files.

### Mode Dwell Time Evaluation in `bcinr-cmca`

The Mode Dwell Time requirement is enforced branchlessly to prevent rapid state oscillation (hysteresis) across two main files: `crates/bcinr-cmca/src/allocator.rs` and `crates/bcinr-cmca/src/mode_switch.rs`. Following the core architectural laws of `bcinr`, it avoids standard conditional branching (like `if` statements) entirely, substituting them with mathematical and bitwise operations.

#### 1. Dwell Token Verification (`mode_switch.rs`)
In `mode_switch.rs`, the system validates the `DwellSatisfied` token to ensure its round and transition identities match the attempted mode switch.

```rust
// crates/bcinr-cmca/src/mode_switch.rs
let dwell_ok = dwell.round_identity() == round_identity
    && dwell.transition_identity() == transition_identity;

let admitted = cert_ok && dwell_ok && state_ok;
```
If `dwell_ok` is false, it produces a typed refusal (`ModeSwitchRefusal::DwellIdentityMismatch`), leaving the state untouched.

#### 2. Branchless Tracking and Threshold Checks (`allocator.rs`)
The core allocation loop computes whether the minimum dwell constraint `tau_d` has elapsed since the last mode switch, creating an update authorization mask rather than branching:

```rust
// crates/bcinr-cmca/src/allocator.rs

// 1. Calculate if enough time has passed:
let can_switch = t.wrapping_sub(local_last_switch_t) >= tau_d;

// 2. Derive whether a switch is wanted, and mathematically combine to determine if updates are allowed:
let switch_wanted = dom_mode != local_prev_mode;
let update_allowed = !(switch_wanted & !can_switch) & !freeze_learning & proof_some;

// 3. Determine if the switch actually goes through:
let did_switch = (new_dom_mode != local_prev_mode) & can_switch & !freeze_learning & proof_some;
```

#### 3. State Update via Mask Selection (`allocator.rs`)
Instead of conditionally mutating state `if did_switch { ... }`, the code builds bitmasks to deterministically select the new state or retain the old state in constant time using `const_select_u32`.

```rust
// crates/bcinr-cmca/src/allocator.rs
local_last_switch_t = const_select_u32(did_switch as u32, t, local_last_switch_t);
local_prev_mode = const_select_u32(did_switch as u32, new_dom_mode, local_prev_mode);
```

#### 4. Constant-Time Selection Implementation (`allocator.rs`)
The `const_select_u32` function transforms the boolean `condition` into an all-ones (`0xFFFFFFFF`) or all-zeros (`0x00000000`) mask using mathematical wrapping and shifts, achieving `CC=1`:

```rust
#[inline(always)]
pub fn const_select_u32(condition: u32, a: u32, b: u32) -> u32 {
    let cond = core::hint::black_box(condition);
    // Expand any non-zero value to 1 in the highest bit, then shift down (handling 1 or >1)
    let cond_val = (cond | cond.wrapping_neg()) >> 31;
    // 0 - 1 = 0xFFFFFFFF, 0 - 0 = 0x00000000
    let mask = 0u32.wrapping_sub(cond_val);
    
    // Select 'a' if mask is all 1s, 'b' if mask is all 0s
    (core::hint::black_box(a) & mask) | (core::hint::black_box(b) & !mask)
}
```

#### 5. Deferred Transactional Commit (`allocator.rs`)
At the end of the allocation loop, persistent state pointers are only updated if no refusals (including `DWELL_UNSATISFIED`) occurred, ensuring bit-for-bit invariance on failure:

```rust
// 'has_refusal' is aggregated earlier via branchless union operations
*last_switch_t = const_select_u32(has_refusal as u32, *last_switch_t, local_last_switch_t);
```

By expressing sequential semantic decisions as polynomials and full-width bitmasks, the authoritative runtime remains fully deterministic, branchless, and mathematically bound.
