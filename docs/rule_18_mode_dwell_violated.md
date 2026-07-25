# ModeDwellViolated Typed Refusal

Based on the internal documentation for the `bcinr` deterministic substrate, here is an explanation of the `ModeDwellViolated` typed refusal mentioned in Rule 18:

## Purpose
The primary purpose of the `ModeDwellViolated` typed refusal is to act as a strict mathematical guard against unstable policy oscillation within the MAPE-K autonomic loop. By enforcing a strict dwell time, it structurally prevents "rapid-switching" or "policy flapping" (e.g., oscillating endlessly between a "learning" and "frozen" mode on every tick). This guarantees thermodynamic and numerical stability across transitions, ensuring the system gathers sufficient telemetry in a specific mode before allowing subsequent autonomic reactions.

## When it is Triggered in the Runtime
It is triggered when an adaptive state attempts to transition to a new control mode before it has persisted in its current mode for a required minimum number of rounds (`MODE_DWELL_ROUNDS_MIN`). 

Due to the project's strict $CC=1$ cyclomatic complexity mandate (the Radon Law) and Rule 9 (Mask-based execution law), this condition is evaluated and the refusal is surfaced completely free of control-flow branches (e.g., avoiding `if elapsed < min { return Err(...); }`).

The runtime triggers this refusal through the following branchless sequence:
1. **Bitwise Comparison**: The elapsed time `tau_d` is compared against `MODE_DWELL_ROUNDS_MIN` using a polynomial branchless comparator (`const_lt_u32`). It extracts the sign bit of the difference without assembly branches, yielding a deterministic `1` (violated) or `0` (satisfied).
2. **Canonical Masking**: The `1` or `0` result is expanded into a full-width canonical mask (where `1` becomes `0xFFFFFFFF` and `0` becomes `0x00000000`). If a violation occurred, the mask enables the logical OR of `RefusalSet::DWELL_UNSATISFIED` into the current refusal set.
3. **Mask-Based State Transition**: The candidate state is either committed or dropped using a bitwise multiplexer (`const_select_u32`). If a dwell violation occurred, the active canonical mask mathematically enforces `(mask & current) | (!mask & candidate)`. The state transition is gracefully reverted bit-for-bit to the existing state, and the `ModeDwellViolated` refusal is surfaced upstream.
