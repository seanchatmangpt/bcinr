# The `EnvelopeViolated` Typed Refusal

In the BCINR deterministic substrate, the `EnvelopeViolated` typed refusal is a critical safety mechanism mandated by Rule 18 of `AGENTS.md`. It structurally enforces the bounded numerical limits defined by the mathematical specifications without violating the substrate's zero-allocation or branchless execution rules (the Radon Law, $CC=1$).

## Structural Circumstances for Triggering

The `EnvelopeViolated` refusal is triggered when an authoritative computation exceeds its mathematically certified boundaries. These bounded constraints—known as the "envelope"—are provided prior to hot-path execution via the `AcceptedEnvelopeReceipt` (as per Rule 11, the ReceiptSound Law). 

The refusal occurs under the following structural circumstances:
- **Numeric Limits Exceeded:** Fixed-point arithmetic operations (e.g., Q16.16) experience an overflow, underflow, or fall outside of declared admissible domains and codomains.
- **Approximation Error Bounds Exceeded:** Evaluated parameters surpass the maximum absolute or relative error boundaries explicitly declared in the mathematical error envelope (e.g., the $\approx 0.08607$ absolute error bound for the piecewise linear $\log_2$ approximation).
- **Capacity Limits Violated:** Structural allocations, such as bumping offsets in the `BumpArena`, exceed fixed deterministic capacities.

## Mathematical Enforcement Without Panics or Control Flow

Because the BCINR runtime strictly forbids panics, `if/else` statements, or early returns, bounds checking and refusal generation must occur through constant-time mathematical evaluations and bitwise masking:

### 1. Generating a Branchless Mask
Instead of using conditional branches, conditions are evaluated mathematically to yield a boolean bit-mask (e.g., `CanonicalMask`). For instance, verifying that no numerical overflow or underflow occurred might use constant-time comparators like `const_eq_u32` or bitwise arithmetic. Success produces a mask of all ones (e.g., `0xFFFFFFFFFFFFFFFF`), and failure yields a mask of all zeros (`0x0`).

### 2. Branchless Fault Accumulation
If the boundaries of the `AcceptedEnvelopeReceipt` are breached, the substrate uses the generated mask to select the appropriate bitwise fault code (e.g., `RANGE_VIOLATION` or `APPROX_ENVELOPE`). The fault is recorded by unioning it with the ongoing state (using a bitwise OR operation like `self.faults = self.faults.union(e)`), ensuring that fault accumulation behaves as a join-semilattice without branching or "first-error-wins" short-circuiting.

### 3. Gating State Mutation
To block the operation and enforce the "No mutation before complete admission" law (Rule 10), state updates are applied via a fixed-width selection function:
```rust
next_state = select(mask, candidate_state, current_state)
```
When the mathematical envelope is violated, the generated mask evaluates to `0`. Consequently, `select(0, candidate, current) = current`. The rejected operation leaves the persistent state bit-for-bit structurally unchanged.

### 4. Signaling at the Boundary
The execution completes its fixed, $O(1)$ constant-time cycle without unwinding. The bitwise fault aggregations are returned opaquely and unwrapped at the substrate boundary (via mappers like `wrap_result`), cleanly manifesting as the `StabilityRefusal::EnvelopeViolated` enum. This safely signals the MAPE-K Autonomic Loop to initiate recovery, successfully blocking the out-of-bounds operation while preserving perfect branchless determinism.
