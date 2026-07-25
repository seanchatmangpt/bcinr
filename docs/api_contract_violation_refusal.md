Here is the documentation on how `ContractViolation` is utilized and produced in the repository.

### Search Findings
Using `grep_search` and `view_file`, I searched for `ContractViolation` in the requested directories. 
- It is **absent** from `crates/bcinr-api/src/`.
- It is actively used in **`crates/bcinr-cmca/src/allocator.rs`**.
- It is also heavily referenced in documentation (`AGENTS.md`, `docs/`) and utilized in `crates/bcinr-powl/src/admit.rs`.

---

### How `ContractViolation` is Produced

`ContractViolation` acts as both a **CI gate concept for testing** and a **runtime typed refusal**, produced structurally without violating the branchless mandates.

#### 1. Pure CI Gate Concept (Hostile Mutation Validation)
In the context of the `@armstrong_fault` testing procedures (e.g., in `crates/bcinr-powl/src/admit.rs`), `ContractViolation` is an explicit test-only failure mode. 

During adversarial testing, deliberate hostile mutants are injected into the branchless DPAG pipeline (e.g., dropping negative sign extensions or swapping priorities). If a mutant produces a result that deviates from the independent, mathematically sound "oracle," the test apparatus structurally halts and returns `Err(StabilityRefusal::ContractViolation)`. This satisfies **Rule 19**, proving the mutant was successfully "killed" for violating a strict axiomatic contract.

#### 2. Runtime Structural Refusal (Branchless Accumulation)
In the authoritative hot path of `crates/bcinr-cmca/src/allocator.rs`, `ContractViolation` is a variant of the public `StabilityRefusal` enum. 

Because the Radon Law ($CC=1$) strictly prohibits data-dependent branches (no `if`, `match`, or `?`), `ContractViolation` cannot be raised via traditional early returns. Instead, it is produced branchlessly:

* **Mask Derivation and Accumulation:** 
  Mathematical constraints and boundaries are checked using fixed-width primitives that yield full bitmasks (e.g., `!0` for valid, `0` for invalid). If a mathematical property (such as round identity) fails, a specific bitflag (like `RefusalSet::ROUND_MISMATCH`) is flipped.
* **Bitwise Aggregation:** 
  These flags are bitwise-unioned (`|`) into a `RefusalSet` struct that bubbles up the call stack.
* **Mask-Based State Rejection:** 
  This cumulative refusal mask is fed directly into branchless multiplexers:
  $$x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t)$$
  If a contract violation occurred, the mask zeroes out the candidate state, leaving persistent state unchanged bit-for-bit (enforcing **Rule 10**).
* **Boundary Translation:** 
  When leaving the strict branchless authoritative boundary, an adapter (`AllocationOutcome::into_result()`) evaluates the `RefusalSet` to extract the `primary_reason()`. Here, if the `RefusalSet` contains the `ROUND_MISMATCH` flag (which signifies a mathematical failure verifying sealed certification bindings upstream), or lacks an explicitly handled bitflag, it translates the branchless error state into the typed error `Err(StabilityRefusal::ContractViolation)`.
