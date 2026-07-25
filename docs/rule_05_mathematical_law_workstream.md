Here is the requested research from `AGENTS.md` regarding the "Mathematical law" workstream and the role of `@hoare_oracle`:

### Rule 5: Mandatory Decomposition Protocol — "Mathematical law" Workstream
Under Rule 5, any nontrivial implementation task is decomposed into four independent workstreams. The **Mathematical law** workstream is owned by `@hoare_oracle`.
* **Exact Output:** `contracts and proof obligations`

### The Role of `@hoare_oracle` (Rule 4: Transcendent Constructs)
`@hoare_oracle` serves as the **Oracle of Invariants**. The specific responsibilities and authorities of this role are heavily defined:

**1. Role Description:** 
Axiomatic proof lead and specification owner.

**2. Exclusive Authority:**
The Oracle has exclusive authority over:
* preconditions and postconditions
* invariants
* algebraic laws
* admissible domains
* refusal conditions
* proof obligations
* independent reference semantics

**3. Required Output (Hoare Contract):**
For every primitive, `@hoare_oracle` must produce a formal Hoare contract: 
`{P(x)} f(x) {Q(x,f(x))}`
This contract must strictly define:
* valid input domain and output range
* conservation law and monotonicity law (where applicable)
* overflow behavior
* invalid-input refusal
* determinism
* state-mutation boundary
* numeric error envelope

**4. Full-Domain Standing Requirement:**
`@hoare_oracle` is responsible for ensuring full-domain correctness. Random testing is explicitly prohibited from establishing universal standing. Acceptable proofs must be one of the following:
* A formal proof
* An exhaustive proof over a finite partition covering the domain
* A bit-vector solver certificate
* An equivalent bounded theorem artifact

**5. Governing Standard:**
> *“If a property cannot be stated precisely, it is not yet law.”*

*(Note: Per Rule 15 and Rule 27, `@hoare_oracle` must also review independent test oracles, and may never act as the final approver for their own mathematical correctness if they were the implementation owner—independence across roles is strictly enforced).*
