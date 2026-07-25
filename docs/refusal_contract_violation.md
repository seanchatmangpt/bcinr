# `ContractViolation` Typed Refusal

## Definition
In the BCINR deterministic substrate, `ContractViolation` is a variant of the `StabilityRefusal` enum. In compliance with **Rule 18 (Typed Refusals)**, it serves as a strict, fixed-width refusal code without using string-based error messages. 

It signifies that a mathematical constraint or axiomatic invariant (the Hoare logic contract: $\{P(x)\} \quad f(x) \quad \{Q(x,f(x))\}$) established by the `@hoare_oracle` has been breached. This implies the input is mathematically inadmissible, or a core conservation law/boundary constraint has failed, and the operation cannot safely satisfy its postcondition without compromising structural integrity.

Additionally, it functions as a **CI gate concept** for adversarial testing (`@armstrong_fault`). If an injected hostile mutant produces a result that deviates from the branchless mathematical oracle (e.g., dropping negative sign extensions or priority inversions), the test apparatus structurally halts and returns `Err(StabilityRefusal::ContractViolation)`.

## Branchless Mathematical Condition
In strict adherence to the **Radon Law ($CC=1$)** and **Rule 8 (Absolute CC=1 Law)**, `ContractViolation` is never triggered via traditional control flow (no `if`, `match`, early returns, or `?`). 

Instead, it is produced branchlessly:
1. **Mask Computation and Bitwise Aggregation:** Preconditions and boundary limits are evaluated using branchless numeric primitives. For example, failing an invariant validation sets specific bitflags (such as `RefusalSet::ROUND_MISMATCH`, which signifies a mathematical failure verifying sealed certification bindings upstream). These flags are bitwise-unioned (`|`) into a `RefusalSet`.
2. **Mask-Based State Rejection:** The cumulative refusal mask determines the state transition via multiplexers:
   $$x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t)$$
   If the mask is `0` (a violation occurred), the candidate state is discarded, and the persistent state remains bit-for-bit unchanged (**Rule 10**).
3. **Boundary Translation:** When leaving the strict branchless authoritative boundary (e.g., `AllocationOutcome::into_result()` in `crates/bcinr-cmca/src/allocator.rs`), the `RefusalSet` is evaluated via `primary_reason()`. If the set contains the `ROUND_MISMATCH` bit (or lacks an explicitly handled bitflag), it translates the branchless error state into the typed error `Err(StabilityRefusal::ContractViolation)`.
