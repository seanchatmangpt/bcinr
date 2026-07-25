### Definition of `ModeDwellViolated`
In the `bcinr` deterministic substrate, `ModeDwellViolated` (internally represented as `RefusalSet::DWELL_UNSATISFIED`) is a typed refusal that serves as a strict mathematical guard against unstable policy oscillation within the MAPE-K autonomic loop.

It ensures that the system persists in its current control mode for a minimum number of rounds (`MODE_DWELL_ROUNDS_MIN`) before attempting to transition to a new mode. By enforcing this strict dwell time, it structurally prevents "rapid-switching" (or "policy flapping"), such as endlessly oscillating between a "learning" and "frozen" mode. This guarantees thermodynamic and numerical stability across transitions, ensuring the system gathers sufficient telemetry before allowing subsequent autonomic reactions. 

### Branchless Mathematical Trigger
In adherence to the Radon Law ($CC=1$) and Mask-based execution (Rule 9), the evaluation of the dwell constraint completely avoids control-flow branches (e.g., `if tau_d < min`).

The elapsed time (`tau_d`) is compared against the minimum rounds (`min`) using a branchless polynomial over two's complement arithmetic to calculate `tau_d < min`:

```rust
// Polynomial branchless `<` comparison
let dwell_err = ((tau_d ^ ((tau_d ^ min) | (tau_d.wrapping_sub(min) ^ min))) >> 31) & 1;
```

This extracts the sign bit of the difference without generating assembly branches, yielding a deterministic `1` (if violated) or `0` (if satisfied).

The deterministic boolean flag is then seamlessly expanded into a canonical mask to conditionally append the refusal without branching:
```rust
let gated_refusals = RefusalSet::EMPTY
    // logically OR the refusal if the mask is 0xFFFFFFFF
    .union(RefusalSet::DWELL_UNSATISFIED.masked(dwell_err as u32));
```

If a violation occurs, the system utilizes a bitwise multiplexer (`const_select_u32`) to block state mutation. The active canonical mask enforces the algebraic rule `(mask & current) | (!mask & candidate)`, reverting the state bit-for-bit to the existing state, and surfacing the `ModeDwellViolated` refusal upstream.
