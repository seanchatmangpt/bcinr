Based on Rule 16 (Anti-cheat manifesto) in `AGENTS.md`, here are the details regarding **CHEAT-004**:

### What constitutes CHEAT-004 (Artificial file inflation)?
Artificial file inflation is defined as the inclusion of:
* Padding
* Repeated comments
* Generated boilerplate
* Dead code 

These elements are specifically added to artificially satisfy line-count or artifact-count expectations.

### Why is it classified as a cheat?
Under the BCINR Deterministic Substrate Constitution, development is governed by strict mathematical contracts, structural compliance, and independent verification rather than superficial metrics. CHEAT-004 is classified as a cheat because:
1. **False Metrics**: It manipulates repository statistics (like line counts or file sizes) to simulate productivity, complexity, or compliance without delivering actual substantive engineering or proof artifacts.
2. **Violation of Contractual Contribution**: Similar to CHEAT-001 ("Self-canceling operations") and CHEAT-005 ("Boilerplate verification claims"), it introduces elements that provide zero contractual contribution to the codebase's output or deterministic execution. 
3. **Substrate Integrity Compromise**: The project demands a perfect Substrate Integrity Score (SIS) based on actual verification (proofs, oracles, hostile mutants). Introducing dead code or padding corrupts the structural purity of the codebase, creates noise that complicates the rigorous source and object-code audits, and fundamentally undermines the project's mission of maintaining a bounded, minimal, and fully deterministic substrate.
