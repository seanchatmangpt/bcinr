I have reviewed `AGENTS.md` and researched the role of `@hoare_oracle` under Rule 4, as well as its specific authority over the "state-mutation boundary."

### Role of `@hoare_oracle` (Rule 4)
Under Rule 4 ("Roster of Transcendent Constructs"), `@hoare_oracle` is defined as the **Oracle of Invariants**. This agent persona serves as the axiomatic proof lead and specification owner for the BCINR deterministic substrate.

The Oracle holds **exclusive authority** over:
* Preconditions, postconditions, and invariants
* Algebraic laws and admissible domains
* Refusal conditions and proof obligations
* Independent reference semantics

For every authoritative primitive, the Oracle is required to output a mathematical Hoare contract: `{P(x)} f(x) {Q(x,f(x))}`. This contract must explicitly specify properties such as the valid input domain, conservation laws, determinism, numeric error envelopes, and the **state-mutation boundary**.

### Authority over the "State-Mutation Boundary"
The "state-mutation boundary" is a strict architectural constraint dictating exactly how and when persistent state can be updated. Because the system's hot path strictly forbids conditional branching (e.g., `if error { rollback() }`) and heap allocations, speculative mutation of state is completely prohibited.

The Oracle uses the state-mutation boundary to mathematically enforce **Rule 10: No mutation before complete admission**. It establishes a rigid transactional shape that must be followed:
1. **Current Immutable State:** Operations begin with the existing state.
2. **Fixed-size Candidate State:** The next potential state is computed structurally on the stack (without heap-backed cloning).
3. **Verify Predicates:** All invariants and admission laws are verified without short-circuiting.
4. **Derive Admission Mask:** A full-width bitmask is generated representing whether the operation is completely admitted.
5. **Fieldwise Masked Commit:** The actual boundary where state mutation occurs. It is executed as a branchless bitwise selection:
   `x_{t+1} = select(m_admitted, x_candidate, x_t)`

By defining this boundary in the mathematical contract, the `@hoare_oracle` guarantees that **a rejected operation leaves the persistent state bit-for-bit unchanged**, and that no partial state mutations ever leak into the runtime.
