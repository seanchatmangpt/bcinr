I have completed the search for `BranchlessContractFailed` in `crates/bcinr-cmca/src/` and `crates/bcinr-api/src/`.

**Findings:**
The exact identifier `BranchlessContractFailed` is actually **absent** from the Rust source code in those directories. My `grep_search` confirmed it only exists in documentation files (like `AGENTS.md` and `docs/branchless_contract_failed_refusal.md`). 

However, in the actual runtime implementation, this exact refusal concept is mapped to the **`StabilityRefusal::ContractViolation`** typed refusal (found in `crates/bcinr-cmca/src/allocator.rs` and the related `crates/bcinr-powl/src/admit.rs`).

Here is the documentation on how this specific typed refusal is produced branchlessly:

### Mathematical Checks and Runtime Validations

The `ContractViolation` (or `BranchlessContractFailed`) refusal is raised when operations deviate from the deterministic mathematical contract. It is structurally enforced without utilizing any control-flow branching ($CC=1$) via the following mechanism:

1. **Formal Hoare Contract Violations:**
   A constraint check evaluates the mathematical requirements. This includes checks like:
   - Ensuring an input parameter remains within a verified fixed-width limit.
   - Verifying round identity matching or minimum urgency thresholds (e.g., via `ge_mask`).
   - Hostile mutants structurally breaking a mathematical law during negative domain testing (`@armstrong_fault`).

2. **Bit-Parallel Accumulation (No Short-Circuiting):**
   Instead of using an early `return`, `unwrap()`, or `panic!` upon failure, the validation evaluates as a boolean constraint that reduces directly to a bitmask (valid = `!0`, invalid = `0`). 
   These failure masks are accumulated purely via bitwise arithmetic into a `RefusalSet`.

3. **Mask-Based State Isolation (`@von_neumann_bypass`):**
   The refusal bitmask mechanically drives mask-based execution logic:
   $$x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t)$$
   If the contract evaluates to the equivalent of a failed mask ($m_{\mathrm{admitted}} = 0$), the candidate mutation is cleanly discarded bit-for-bit, keeping persistent state intact.

4. **Envelope Boundary Translation (`@turing_machine`):**
   The enforcement gates demand that internal state remains flat and bitwise until it hits a strict Envelope Boundary. At this boundary, an adapter (such as `RefusalSet::primary_reason()` in `crates/bcinr-cmca/src/allocator.rs`) mechanically translates the accumulated bitmask into the typed error `Result::Err(StabilityRefusal::ContractViolation)`. This entirely avoids conditional jumps in the hot path while securely signaling the failure up the stack.
