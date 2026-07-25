# The Seven Elements of Authoritative Primitives

According to the **BCINR Deterministic Substrate Constitution** (Rule 1: Mission), a feature is never considered complete based merely on the fact that it "appears correct in tests." Instead, every authoritative primitive must possess the following seven explicit elements:

1. **A mathematical contract:** A rigorously defined Hoare contract mapping preconditions to postconditions and specifying invariant behaviors.
2. **A structurally lawful implementation:** Code that adheres to absolute runtime laws (e.g., $CC=1$, branchless, allocation-free, fixed bounded execution).
3. **An independent oracle or proof:** A mathematically separate specification (not a mirror of the implementation) used as a baseline for correctness.
4. **Hostile mutants:** Syntactically plausible, adversarial mutations of the implementation designed to explicitly verify typed refusals and safety bounds.
5. **Source-level verification:** Source-code level scanning (via tools like the `bcinr-cheat-scanner`) enforcing cyclomatic complexity of 1 and verifying structural integrity.
6. **Object-code verification:** Exact production-profile disassembly audits proving the compiled machine code has no branches, loop backedges, or runtime abstractions.
7. **Reproducible evidence:** A mechanical artifact (e.g., a standing receipt or verifiable matrix) proving that all verifications passed successfully.

## The Driving Principle

The fundamental governing principle behind these requirements is: **"Rich semantics upstream. Fixed deterministic mechanics downstream."** 

The mission of BCINR is to serve as a deterministic computational substrate for bounded, branchless, allocation-free execution. To uphold this mission, the authoritative runtime must guarantee an unbroken chain: 

**Admitted Input $\rightarrow$ Fixed Instruction Shape $\rightarrow$ Deterministic Output**

Relying on standard unit tests is insufficient because tests only sample runtime behavior. By demanding mathematical proofs, independent oracles, hostile mutation, and physical object-code verification, the project ensures that the actual hardware instructions executed are entirely independent of semantic inputs, achieving true determinism and eliminating timing side-channels at the substrate level.
