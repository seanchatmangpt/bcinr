# The Concept of "Admitted Control State" in BCINR

In the context of the BCINR deterministic computational substrate, the concept of an "Admitted Control State" is central to maintaining the rigid state-mutation boundary defined by the project's constitutional laws (specifically Rule 10 and Rule 11 of `AGENTS.md`). It operates strictly within the MAPE-K autonomic loop to guarantee deterministic, allocation-free, and branchless execution.

## Speculative Candidate State vs. Admitted State

To understand the "Admitted Control State," it is essential to distinguish it from a "Speculative Candidate State."

### Speculative Candidate State
A **Speculative Candidate State** is a transient, intermediate data structure computed on the stack (without heap allocations). It represents the *potential* next state of the system after applying a set of inputs or actions. However, at this stage, it is entirely unverified. It has not yet been checked against system invariants, policy guards, or mathematical contracts.

### Admitted Control State
An **Admitted Control State** is a candidate state that has successfully passed all rigorous mathematical predicate verifications and policy guards. It acts as a structural proof—a required receipt—that the proposed state transition is lawful. Once a state is "admitted," an admission mask is derived, mathematically authorizing the transition from the current state to the new state. 

In the context of **Rule 11 (The ReceiptSound Law)**, adaptive mutation is strictly gated. The runtime cannot modify persistent state unless it holds an irrefutable combination of structural proofs, including the `AdmittedControlState` (alongside `AcceptedCertificate`, `AcceptedEnvelopeReceipt`, `AcceptedOutcomeReceipt`, and `CertifiedLearningMode`). No alternate constructors or APIs are permitted to bypass this requirement.

## Integration with the MAPE-K Autonomic Loop

The transition from a speculative candidate to an admitted control state maps directly onto the BCINR implementation of the MAPE-K (Monitor, Analyze, Plan, Execute, Knowledge) autonomic loop:

1. **Observe**: The system collects bit-level telemetry and deterministic inputs.
2. **Infer (Analyze)**: The system calculates the current status (e.g., `RlState`) using branchless metrics.
3. **Propose (Plan)**: The system generates `AutonomicAction` masks and computes a **Speculative Candidate State**. This calculation is done purely in fixed-size scratch space on the stack.
4. **Accept**: The candidate state is filtered through the `PolicyGuard` and all Hoare contract predicates are verified. If all checks pass, the system derives the admission mask, effectively generating the **Admitted Control State** and satisfying Rule 11.
5. **Execute**: The system advances the persistent state via constant-time transitions using the derived masks.

## Connection to Rule 10 (No mutation before complete admission)

Rule 10 strictly forbids partial, branching, or speculative mutation of persistent state (e.g., modifying state and subsequently returning an `Err()` if a later check fails). Persistent state must be updated atomically and deterministically.

The concept of the `AdmittedControlState` enforces Rule 10 through a rigid transaction pipeline:
`current immutable state` $\rightarrow$ `fixed-size candidate state` $\rightarrow$ `verify all predicates` $\rightarrow$ `derive admission mask` $\rightarrow$ `fieldwise masked commit`

By separating the generation of the candidate state from the actual commit phase, BCINR ensures that the persistent state is never touched until complete admission is guaranteed. 

The final commit step across the state-mutation boundary is executed via a branchless selection:
$$x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t)$$

If the candidate state is rejected (i.e., it never becomes an `AdmittedControlState`), the admission mask $m_{\mathrm{admitted}}$ resolves to $0$. The branchless selection mathematically guarantees that the state falls back to $x_t$, leaving the persistent state bit-for-bit unchanged, fully satisfying the requirements of Rule 10.
