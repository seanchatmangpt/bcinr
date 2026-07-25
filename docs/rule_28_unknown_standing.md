### `UNKNOWN` Standing in Rule 28

**Definition from Rule 28:** 
> **`UNKNOWN`**
> 
> Evidence is insufficient.

**Conditions for receiving this standing:**
A piece of code is labeled as `UNKNOWN` when it lacks the rigorous, required mechanical evidence to prove its compliance with the strict BCINR constitutional rules. 

Specifically, this standing applies under conditions such as:
1. **Missing Evidence Artifacts:** The code is missing one or more of the mandatory verification artifacts required by Rule 29 (such as `CONTRACT.md`, `SOURCE_AUDIT.md`, `OBJECT_CODE_AUDIT.md`, `MUTANT_KILL_MATRIX.md`, etc.). Under the constitution, "Claims made outside these artifacts have no standing."
2. **Incomplete Checkpoints:** The implementation process (Rule 30) has not completed all required checkpoints. A feature is not considered verified until it has an independent oracle, passed source gates (like `CC=1` and `bcinr-cheat-scanner`), defeated hostile mutants, and passed an object-code disassembly audit.
3. **Unsubstantiated Agent Claims:** An agent makes a subjective claim (e.g., "looks correct," "should be branchless," "appears safe" - which are explicitly prohibited in Rule 31) without backing it up with specific bounded claims and linked evidence. As per Rule 27, "Agent agreement is not evidence."

In short, any code that has not yet successfully passed the entire verification matrix and produced the corresponding indisputable artifacts has an `UNKNOWN` standing.
