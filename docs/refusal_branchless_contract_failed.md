### Definition
**`BranchlessContractFailed`** is a strictly bounded **Typed Refusal** that serves as the structural fail-safe for the `bcinr` deterministic runtime's axiomatic calculus. It guarantees zero-allocation boundaries and enforces the absolute Radon Law ($CC=1$).

It acts as an explicitly typed error state returned when operations deviate from the deterministic mathematical contract. Instead of short-circuiting with an early `return`, branching, or panicking, the refusal is mechanically translated at a strict Envelope Boundary to avoid conditional jumps in the hot path.

### Branchless Mathematical Condition
The condition that triggers the refusal is evaluated strictly as a bitmask based on formal Hoare contracts. 

The mathematical condition requires that valid requirements evaluate to a boolean constraint reduced to a bitmask. When a constraint fails (e.g., parameter exceeds fixed-width limits), it evaluates to a mask of `0`:
$$m_{\mathrm{admitted}} = 0$$

These masks are accumulated into a `RefusalSet` and resolve to the `BranchlessContractFailed` state. This mask directly drives the mask-based state isolation logic:
$$x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t)$$

When $m_{\mathrm{admitted}} = 0$, the candidate mutation ($x_{\mathrm{candidate}}$) is discarded bit-for-bit and the persistent state ($x_t$) remains perfectly intact, preventing speculative mutation and ensuring that mathematical rejection never necessitates control-flow rejection.

This behavior strictly enforces the rule that falling back to simpler, branching, or floating-point algorithms is prohibited when an optimized branchless constraint fails.
