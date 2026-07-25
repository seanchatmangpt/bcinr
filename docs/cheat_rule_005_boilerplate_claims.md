# CHEAT-005: Boilerplate Verification Claims

## Definition
According to Rule 16 in the `AGENTS.md` constitution, **CHEAT-005 (Boilerplate verification claims)** prohibits the use of repeated comments asserting verification without a linked proof or receipt.

## Why are these claims prohibited?

The prohibition of boilerplate verification claims stems directly from the core tenets of the BCINR Deterministic Substrate Constitution. Here are the primary reasons why asserting verification in comments is considered an anti-pattern and an enforceable violation:

### 1. Source Claims Do Not Substitute for Evidence
The constitution explicitly states: *"Source claims do not substitute for disassembly evidence."* 
Writing `// Verified branchless` or `// Mathematical contract satisfied` inside source code provides no cryptographic or deterministic proof that the property holds. The system requires reproducible evidence, not human-readable text.

### 2. Agent/Human Agreement Is Not Evidence
As defined in the self-certification laws (Rule 27): *"Agent agreement is not evidence. Five agents repeating the same claim is still one unsupported claim."* 
A comment is merely a claim made by an author (human or agent). An actual proof must be backed by a mechanical artifact, such as an independent oracle, a Hoare contract, or a bit-vector solver certificate. 

### 3. Requirement for Linked Proofs and Receipts
Every authoritative implementation in BCINR must have seven pillars of verification, including:
- An independent oracle or proof
- Source-level verification
- Object-code verification
- Reproducible evidence

When verification is claimed, it must be accompanied by a tangible link (such as a receipt digest or proof artifact) that mechanically binds the source to its verification output. Without this receipt, the claim is treated as an unsubstantiated assertion or "Boilerplate verification."

### 4. Human-Readable Text Belongs Outside the Hot Path
The authoritative hot path is meant purely for fixed deterministic mechanics. Extraneous comments that attempt to inject semantic assertions into the source tree circumvent the strict requirement that *"Rich semantics upstream"* must be mechanically lowered into *"Fixed deterministic mechanics downstream."* 

## The Consequence
Under Rule 24, fabricating verification evidence (which includes making unsupported claims of verification) is an **absolute failure**. It instantly forces the Substrate Integrity Score (SIS) to 0 and triggers a MaturityScrutiny protocol, freezing feature development until the violation is resolved and mechanical proof is produced.
