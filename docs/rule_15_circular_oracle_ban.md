Based on `AGENTS.md`, here is the breakdown of why a "line-by-line translation of production code" and the "reuse of production lookup tables" are strictly prohibited under Rule 15 (Independent oracle law) and categorized as a "circular oracle" (CHEAT-002):

### 1. The Definition of a Circular Oracle (CHEAT-002)
Rule 16 defines a "circular oracle" (CHEAT-002) as "A reference implementation copied from the production implementation." It is a test or reference that self-certifies by merely asking the code to confirm its own logic, rather than verifying against an independent mathematical standard.

### 2. Loss of Independence and Mirroring Flaws
The core mandate of the **Independent oracle law** is that the oracle must verify the primitive's Hoare contract (its mathematical specification) independently. 
* **Line-by-line translation:** If an oracle is just a literal translation of the production code (for example, porting the Rust implementation into Python or mimicking it with `f64`), any logic error, missed edge case, or flawed assumption in the production code will be perfectly mirrored in the test. 
* **Reuse of lookup tables:** Similarly, if a test relies on the exact same lookup tables or fixed-point helpers used in production, it is implicitly trusting the production data rather than verifying it. If the lookup table was generated incorrectly, the oracle will blindly approve the incorrect output.

### 3. The Requirement for Distinct Structure
Because of these risks, the BCINR constitution strictly prohibits these practices. An oracle is not independent simply because it resides in a `tests/` directory. To be structurally and logically distinct, an oracle must take an alternative, fundamentally independent path to verify the postconditions. Permitted forms include:
* Direct mathematical formulas
* Hoare specifications or abstract state machines
* Symbolic proofs
* Arbitrary-precision implementations
* SAT/SMT bit-vector models or exhaustive enumerators

In summary, a line-by-line translation or reused table creates a **tautology** ("the code does what the code does") rather than a rigorous verification ("the code satisfies the invariant mathematical law"). Because it fails to independently challenge the implementation, it is classified as a cheat and strictly banned.
