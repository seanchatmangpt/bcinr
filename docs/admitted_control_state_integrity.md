# Admitted Control State Integrity in BCINR

In the BCINR deterministic computational substrate, the concept of the `AdmittedControlState` is a central requirement under **Rule 11 (ReceiptSound Law)** of the `AGENTS.md` constitution. It ensures that the runtime cannot modify persistent state during adaptive mutation unless it holds irrefutable structural proofs.

## What does `AdmittedControlState` encapsulate?

The `AdmittedControlState` is not just a standard data structure—it acts as an undeniable structural proof (a receipt) that a proposed state transition is mathematically lawful. It encapsulates a candidate state that has successfully passed all rigorous mathematical predicate verifications and policy guards. 

Under the **ReceiptSound Law (Rule 11)**, adaptive mutation is strictly gated. The `AdmittedControlState` forms one part of a required combination of receipts (alongside `AcceptedCertificate`, `AcceptedEnvelopeReceipt`, `AcceptedOutcomeReceipt`, and `CertifiedLearningMode`). No alternate constructors or APIs are permitted to bypass this requirement. Once a state is formally "admitted," an admission mask is derived, mathematically authorizing the transition from the current state to the new state.

## Ensuring the Integrity of the Control Plane

To prevent tampering or speculative mutation before participating in the adaptive mutation equation, the system relies on **Rule 10 (No mutation before complete admission)** and the branchless properties of the MAPE-K autonomic loop. 

1. **Stack-Based Speculative Calculation:**
   During the "Propose" phase, the system generates a **Speculative Candidate State**. This calculation is transient, executed entirely in fixed-size scratch space on the stack without any heap allocations. At this stage, the persistent state remains strictly untouched.

2. **Rigid Transaction Pipeline:**
   The candidate state must pass through a rigid, sequential transaction pipeline before any mutations can occur:
   `current immutable state` $\rightarrow$ `fixed-size candidate state` $\rightarrow$ `verify all predicates` $\rightarrow$ `derive admission mask` $\rightarrow$ `fieldwise masked commit`

3. **Branchless Masked Commit:**
   The final commit step across the state-mutation boundary operates strictly via a branchless selection:
   $$x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t)$$
   If any predicate fails, the candidate state never becomes an `AdmittedControlState`, and the admission mask ($m_{\mathrm{admitted}}$) resolves to $0$. The branchless selection mathematically guarantees that the operation falls back to the current state ($x_t$), leaving the persistent state bit-for-bit unchanged. 

By physically decoupling the generation of the candidate state from the commit phase and utilizing branchless arithmetic for the final transition, BCINR structurally guarantees that the control plane cannot be speculatively mutated or tampered with before admission.
