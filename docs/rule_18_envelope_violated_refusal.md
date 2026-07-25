Here is the research on the `EnvelopeViolated` typed refusal mentioned in Rule 18 of `AGENTS.md`:

### What is the `EnvelopeViolated` Typed Refusal?
The `EnvelopeViolated` typed refusal (`StabilityRefusal::EnvelopeViolated` or `RuntimeEnvelopeViolated`) is a critical safety mechanism in the BCINR deterministic substrate. It structurally enforces the bounded numerical limits and mathematically certified boundaries (the "envelope") defined by mathematical specifications without violating the substrate's zero-allocation or branchless execution rules (the Radon Law, $CC=1$). These boundaries are provided prior to hot-path execution via the `AcceptedEnvelopeReceipt` (as mandated by Rule 11, the ReceiptSound Law).

### When is it surfaced in the runtime?
The refusal is triggered under the following structural circumstances when an authoritative computation exceeds its defined bounds:
1. **Numeric Limits Exceeded:** Fixed-point arithmetic operations (such as Q16.16) experience an overflow, underflow, or fall outside of declared admissible domains and codomains.
2. **Approximation Error Bounds Exceeded:** Evaluated parameters surpass the maximum absolute or relative error boundaries explicitly declared in the mathematical error envelope (e.g., exceeding the $\approx 0.08607$ absolute error bound for the piecewise linear $\log_2$ approximation).
3. **Capacity Limits Violated:** Structural allocations, such as bumping offsets in the `BumpArena`, exceed fixed deterministic capacities.

### How is it evaluated in a branchless substrate?
Because the BCINR runtime strictly forbids panics, `if/else` statements, or early returns, bounds checking and refusal generation must occur through constant-time mathematical evaluations and bitwise masking:

1. **Branchless Mask Generation:** Conditions are evaluated mathematically to yield a boolean bit-mask (`CanonicalMask`). Success produces a mask of all ones (e.g., `0xFFFFFFFFFFFFFFFF`), and failure yields a mask of all zeros (`0x0`).
2. **Branchless Fault Accumulation:** Using the generated mask, the algorithm selects the appropriate bitwise fault code (e.g., `RANGE_VIOLATION` or `APPROX_ENVELOPE`). The fault is unioned with the ongoing state using a bitwise OR operation, operating as a join-semilattice without branching or "first-error-wins" short-circuiting.
3. **State Masking (Gating Mutation):** In accordance with the "No mutation before complete admission" law (Rule 10), state updates are applied via a fixed-width selection function: `next_state = select(mask, candidate_state, current_state)`. If the mathematical envelope is violated, the generated mask evaluates to `0`, ensuring the rejected operation leaves the persistent state bit-for-bit unchanged.
4. **Signaling at the Boundary:** The operation completes its fixed $O(1)$ cycle time without unwinding. The bitwise fault aggregations are unwrapped at the substrate boundary by mappers (e.g., `wrap_result`), safely converting the bitwise mask into the typed `StabilityRefusal::EnvelopeViolated` enum. This cleanly signals the MAPE-K Autonomic Loop to initiate recovery while preserving branchless determinism.
