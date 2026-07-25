## CHEAT-010 — Gate-jurisdiction theater

**What is it?**
Under Rule 16 of the BCINR Deterministic Substrate Constitution, **CHEAT-010 (Gate-jurisdiction theater)** is defined as:
> "Reporting a passing scanner that does not inspect the relevant crate, file, generated output, feature set, or target."

This occurs when an agent runs a verification tool or scanner and reports a success ("green command"), but the tool was executed in a way that bypassed or excluded the actual code changes, files, or configurations that needed to be audited.

**Why is checking jurisdiction required before reporting results?**
Checking jurisdiction is strictly required to ensure that the verification tools are actually analyzing the relevant modifications rather than running against an unaffected or stubbed environment. According to the constitution (specifically Rules 23, 24, and 32):

1. **Green is not enough:** The constitution states, "A green command with incomplete jurisdiction is not evidence." Just because a tool succeeds does not mean the code is mathematically and structurally sound if the changed files weren't inspected.
2. **Burden of Proof:** Before reporting any results, you must explicitly *prove* that each verification task's jurisdiction includes the changed files. The final report must explicitly list the `files inspected`, `features inspected`, and `targets inspected`.
3. **Absolute Failure:** A "gate-jurisdiction omission" is classified as an absolute failure that forces the Substrate Integrity Score (SIS) to `0`, bypassing any weighted averages and immediately triggering the `MaturityScrutiny` protocol. 
4. **Immediate Agent Purge:** Excluding changed files from scanner jurisdiction is an explicit violation that results in the agent being immediately removed from the active task.

In short, jurisdiction verification is required because the BCINR project demands cryptographic-level certainty, and passing a test that didn't look at the code is treated as fabricated evidence.
