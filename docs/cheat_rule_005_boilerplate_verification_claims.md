# CHEAT-005 — Boilerplate Verification Claims

## Definition
**CHEAT-005** occurs when an agent or author includes comments, documentation, or annotations asserting that a verification step was completed, without linking to a reproducible mechanical proof, artifact digest, or receipt.

## What Constitutes This Cheat?
In the BCINR deterministic substrate, the following patterns constitute CHEAT-005:
- Inserting comments such as `// Verified branchless`, `// Passed CC=1 audit`, or `// Mutants killed` without a corresponding verifiable receipt digest.
- Copy-pasting boilerplate verification claims across multiple files to bypass human or scanner scrutiny.
- Repeating assertions of mathematical correctness or Hoare contract satisfaction without pointing to the independent oracle or symbolic proof artifact.
- Claiming an object-code audit was performed without the exact production-profile disassembly evidence (e.g., the table mapping symbols to conditional jumps and loop backedges).

## Why is it Strictly Prohibited in the Deterministic Substrate?
The governing principle of the BCINR repository is **"rich semantics upstream, fixed deterministic mechanics downstream."** Therefore, hearsay is not recognized as truth. 

1. **Agent Agreement is Not Evidence:** As outlined in Rule 27, an agent's assertion—or even five agents repeating the same claim—is still considered one unsupported claim. Evidence must be a mechanical artifact.
2. **Fabricated Verification Triggers Absolute Failure:** A core law of the substrate is that any fabricated verification evidence forces a Substrate Integrity Score (SIS) of 0 and triggers `MaturityScrutiny`. Allowing empty assertions compromises the hard guarantees of the substrate.
3. **Prevention of Verification Theater:** Permitting boilerplate text allows features to merely "appear" correct. The BCINR mandate requires that a feature is not complete until all seven artifacts (mathematical contract, lawful implementation, independent oracle, hostile mutants, source verification, object-code verification, reproducible evidence) exist.
4. **Mechanical Tooling Requirements:** The `bcinr-cheat-scanner` and autonomic loop require cryptographically verifiable receipts (e.g., `ReceiptSound` law) to process state. Textual assertions cannot be mathematically parsed or verified by the `turing_machine` enforcer.

If a property cannot be stated precisely and backed by an independent, reproducible artifact, it is not yet law.
