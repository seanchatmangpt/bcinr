# The "Contract with Teeth" in BCINR

In the `bcinr` (BranchlessCInRust) project, the **"Contract with Teeth"** is a foundational architectural law governing the reliability and mathematical safety of the deterministic computational substrate. It dictates that the codebase does not accept implementations that merely "appear correct in tests." Instead, every primitive is subjected to an unforgiving, mathematically proven verification process.

## What Does It Mean That Every Primitive Is an "Executable Specification"?

In `bcinr`, a primitive is not just code written to satisfy a feature request; it is an **executable specification**. This means:

1. **Formal Hoare Contracts:** Instead of just written documentation, every primitive has a mathematical Hoare contract ($P(x) \rightarrow f(x) \rightarrow Q(x, f(x))$). This contract explicitly dictates the valid input domain, output range, conservation laws, numeric error envelopes, and invalid-input refusals.
2. **Beyond Standard Execution:** The specification is actively enforced by an independent "oracle" (`@hoare_oracle`) via symbolic logic, SMT/SAT bit-vector solvers, or exhaustive reduced-domain enumerators. 
3. **Multi-Faceted Verification:** The specification requires a structurally lawful branchless implementation ($CC=1$, zero allocation), source-level syntax audits, object-code disassembly verification, and adversarial hostile mutation testing (`@armstrong_fault`). 
4. **Behavioral Rigidity:** The execution must refuse invalid state transactionally and deterministically, throwing exact bounded typed refusals (e.g., `ContractViolation`, `NumericRangeExceeded`) rather than panicking or producing silently corrupted outputs. 

In short, the code *is* the math. The specification is executed and continuously proven against the implementation via the build matrix.

## The Axiomatic Reference and the 1-Bit Deviation Rule

The **axiomatic reference** is the independent oracle. It is a structurally and logically distinct mathematical model—often using arbitrary precision or unoptimized logic—that defines the absolute truth of the primitive's behavior. The production implementation, heavily optimized with SWAR (SIMD Within A Register), masking, and branchless arithmetic, must match this oracle exactly.

### Why MUST the Verification Matrix Fail for a 1-Bit Deviation?

If the highly optimized, branchless implementation deviates from the axiomatic reference by **even 1 bit**, the verification matrix strictly fails for several critical reasons:

1. **Loss of Determinism:** `bcinr` is mandated to be a "civilizational-scale systems library providing an axiomatic calculus for branchless algorithmics" (intended for AGI). Determinism is absolute. A 1-bit deviation means the state transition function is no longer mathematically isomorphic to the proof.
2. **Cascading State Corruption:** In the Autonomic Loop (MAPE-K), bit-level metrics drive the `RlState` (Reinforcement Learning State) and `AutonomicAction` masks. A single flipped bit in an arithmetic mask can flip a deterministic decision, leading to divergent and uncertified state mutations downstream.
3. **Invalidation of the Hoare Contract:** The mathematical proof (e.g., maximum absolute error, monotonicity, saturation behavior) applies *only* to the exact axiomatic reference. A 1-bit deviation voids the proof, making the runtime's behavior officially undefined and untrustworthy.
4. **Substrate Integrity Score (SIS) Violation:** Any deviation from the oracle means the implementation fails the `MaturityScrutiny` protocol. A verified 1-bit failure forces `SIS = 0`, freezing feature development and quarantining the affected code until the structural defect is repaired and reproducible evidence is generated.

In `bcinr`, there are no "close enough" approximations or hidden compiler optimizations. A 1-bit difference proves that the optimization has compromised the foundational mathematical safety of the substrate, triggering an immediate and un-bypassable gate failure.
