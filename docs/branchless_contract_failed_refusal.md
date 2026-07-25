# The `BranchlessContractFailed` Typed Refusal

In the `bcinr` deterministic substrate, `BranchlessContractFailed` is a strictly bounded **Typed Refusal** that serves as the structural fail-safe for the runtime's axiomatic calculus. It guarantees zero-allocation boundaries and enforces the absolute Radon Law ($CC=1$).

## Structural and Verification Circumstances for Raising the Refusal

This refusal is raised when operations deviate from the deterministic mathematical contract without utilizing control-flow branching:

1. **Mathematical Contract Violations (`@hoare_oracle`):** 
   When a formal Hoare contract—which defines valid input domains, conservation laws, and exact refusal conditions—is violated. For example, if a parameter exceeds its verified fixed-width limit, the required condition fails.
2. **Bit-Parallel Accumulation:**
   Instead of short-circuiting with an early `return` or `panic!`, the failure evaluates as a boolean constraint reduced to a bitmask (where necessary mathematical requirements evaluate to `0`). These masks are accumulated seamlessly into a `RefusalSet`, eventually resolving explicitly as the `BranchlessContractFailed` refusal state.
3. **Silent Fallback Avoidance (CHEAT-022):** 
   If an edge case or unsupported condition is encountered, the runtime is strictly prohibited from bypassing primary branchless fixed-point constraints by falling back to simpler, branching, or floating-point algorithms. If an optimized constraint fails, the runtime *must* yield `BranchlessContractFailed`.
4. **Envelope Boundary Translation (`@turing_machine`):** 
   The enforcement gates demand that internal state remains flat and bitwise until it hits a strict Envelope Boundary. At this boundary, an adapter mechanically translates the accumulated `RefusalSet` bitmask into `Result::Err(StabilityRefusal::BranchlessContractFailed)` for the caller, entirely avoiding conditional jumps in the hot path.

## Protecting the Substrate During Constraint Failures

If an operation is detected to violate the deterministic mathematical contract, `BranchlessContractFailed` protects the integrity of the substrate through strict mechanical enforcement:

1. **Guaranteeing Branchless Execution ($CC=1$):** 
   By ensuring that *mathematical rejection never necessitates control-flow rejection*, the refusal protects the runtime from conditional branches. Any attempt to handle a broken constraint via `if !valid` or `unwrap()` is immediately flagged as a violation by the object-code disassembler and `bcinr-cheat-scanner`, blocking the merge.
2. **Mask-Based State Isolation (`@von_neumann_bypass`):** 
   The refusal state mechanically drives mask-based execution logic:
   $$x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t)$$
   If the Hoare specification evaluates to a `BranchlessContractFailed` equivalent mask ($m_{\mathrm{admitted}} = 0$), the candidate mutation is discarded bit-for-bit. Persistent state is never overwritten speculatively, remaining perfectly intact while safely signaling the refusal up the stack.
3. **Hostile Mutant Verification (`@armstrong_fault`):** 
   When negative domain testing injects mutants that structurally break mathematical laws (like skipping a mask or truncating a bounding table), the test protocol demands typed proof of failure. The test MUST explicitly assert `assert_eq!(result, Err(StabilityRefusal::BranchlessContractFailed))`. Generic `assert_ne!` or panic-checks are prohibited. If this strict typed refusal is bypassed, the Substrate Integrity Score (SIS) drops to `0`, halting feature work until integrity is restored.
