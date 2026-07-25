# ModeDwellViolated Typed Refusal

In the `bcinr` deterministic substrate, the `ModeDwellViolated` typed refusal (enforced under Rule 18) acts as a strict mathematical guard against unstable policy oscillation within the MAPE-K autonomic loop.

## 1. What constitutes a mode dwell violation?

A mode dwell violation occurs when an adaptive state within the MAPE-K loop attempts to transition to a new control mode before it has persisted in its current mode for a required minimum number of rounds (`MODE_DWELL_ROUNDS_MIN`). The MAPE-K loop dictates state transitions through its Observe-Infer-Propose-Accept-Execute cycle. If a proposed mode change is evaluated before the elapsed dwell rounds (`tau_d`) equal or exceed the minimum threshold, it constitutes a violation, and the transition must be safely refused.

## 2. Preventing Rapid-Switching and Policy Flapping

By enforcing a strict dwell time, the `ModeDwellViolated` refusal structurally prevents rapid-switching (or "policy flapping") where the system oscillates endlessly between competing states (e.g., switching between a "learning" and "frozen" mode on every tick). This guarantees thermodynamic and numerical stability across transitions, ensuring that the system gathers sufficient telemetry in a specific mode before allowing subsequent autonomic reactions. If violated, the state mathematically drops the candidate mode and bit-for-bit preserves the existing state, inherently maintaining stability.

## 3. Branchless Calculation and Refusal

Following the Radon Law ($CC=1$) and Rule 9 (Mask-based execution), the hot path evaluates the dwell constraint and applies the refusal completely free of control-flow branches (e.g., avoiding `if elapsed < min { return Err(...); }`).

### Bitwise Comparison
The elapsed time `tau_d` is compared against `MODE_DWELL_ROUNDS_MIN` using `const_lt_u32`. This primitive calculates `tau_d < min` using a proven bitwise polynomial over two's complement arithmetic, extracting the sign bit of the difference without assembly branches:
```rust
// Polynomial branchless `<` comparison
let dwell_err = ((tau_d ^ ((tau_d ^ min) | (tau_d.wrapping_sub(min) ^ min))) >> 31) & 1;
```
This yields a deterministic `1` (if violated) or `0` (if satisfied).

### Canonical Masking
The `1` or `0` result is expanded into a full-width canonical mask (where `1` becomes `0xFFFFFFFF` and `0` becomes `0x00000000`). This mask is logically OR'd (`union`) into the current refusal set:
```rust
let gated_refusals = RefusalSet::EMPTY
    // logically OR the refusal if the mask is 0xFFFFFFFF
    .union(RefusalSet::DWELL_UNSATISFIED.masked(dwell_err as u32));
```

### Mask-Based State Transition
Finally, the candidate state is either committed or dropped using a bitwise multiplexer (`const_select_u32`), completely eliminating `if !has_refusal` execution blocks:
```rust
*persistent = const_select_u32(has_refusal as u32, *persistent, candidate);
```
If a dwell violation occurred, the active canonical mask mathematically enforces `(mask & current) | (!mask & candidate)`. The transition gracefully reverts to the `current` persistent state, and the `ModeDwellViolated` typed refusal is safely surfaced upstream.
