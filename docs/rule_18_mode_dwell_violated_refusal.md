I have researched the `ModeDwellViolated` typed refusal mentioned in Rule 18 of `AGENTS.md`. Here is the markdown detailing what it is and when it is surfaced in the runtime:

# ModeDwellViolated Typed Refusal

In the `bcinr` deterministic substrate, the `ModeDwellViolated` typed refusal acts as a strict mathematical guard against unstable policy oscillation within the MAPE-K autonomic loop.

## What it is
The `ModeDwellViolated` refusal prevents "rapid-switching" or "policy flapping" (e.g., oscillating endlessly between "learning" and "frozen" modes). By enforcing a strict dwell time, it guarantees thermodynamic and numerical stability across transitions. This ensures that the system gathers sufficient telemetry in a specific mode before allowing subsequent autonomic reactions. If a violation occurs, the state mathematically drops the candidate mode and bit-for-bit preserves the existing state to maintain stability.

## When it is surfaced in the runtime
It is triggered when an adaptive state attempts to transition to a new control mode before it has persisted in its current mode for a required minimum number of rounds (`MODE_DWELL_ROUNDS_MIN`).

Due to the project's strict $CC=1$ cyclomatic complexity mandate (the Radon Law) and Rule 9 (Mask-based execution), this condition is evaluated and the refusal is surfaced completely free of control-flow branches (avoiding conditionals like `if elapsed < min { return Err(...); }`). 

The runtime surfaces this refusal through the following branchless sequence:
1. **Bitwise Comparison**: The elapsed time `tau_d` is compared against `MODE_DWELL_ROUNDS_MIN` using a polynomial branchless comparator (`const_lt_u32`). It calculates `tau_d < min` using a proven bitwise polynomial over two's complement arithmetic, isolating the sign bit of the difference. This yields a deterministic `1` (violated) or `0` (satisfied).
2. **Canonical Masking**: The `1` or `0` result is expanded into a full-width canonical mask (`0xFFFFFFFF` or `0x00000000`). If a violation occurred, the mask logically ORs (`union`) the `DWELL_UNSATISFIED` bit into the current refusal set.
3. **Mask-Based State Transition**: The candidate state is committed or safely dropped using a bitwise multiplexer (`const_select_u32`). If a dwell violation occurred, the active canonical mask mathematically enforces `(mask & current) | (!mask & candidate)`. The state transition is gracefully reverted to the existing state, and the `ModeDwellViolated` refusal is safely surfaced upstream.
