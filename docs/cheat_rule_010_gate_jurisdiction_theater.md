### What is CHEAT-010?
In Rule 16 (Anti-cheat manifesto), **CHEAT-010 — Gate-jurisdiction theater** is defined as:
> *Reporting a passing scanner that does not inspect the relevant crate, file, generated output, feature set, or target.*

### Why is it considered a failure?
In the BCINR determinism constitution, a green build or a passing scanner is completely meaningless if the scope of the inspection is incomplete. A passing scanner that misses relevant targets is a failure for several constitutional reasons:

1. **"A green command with incomplete jurisdiction is not evidence" (Rule 23)**: If the scanner doesn't inspect the changed files, generated code, or specific feature sets, it is providing a false sense of security. The project requires structural, bit-level, and branchless enforcement across the *entire* authoritative call graph.
2. **Object Code Divergence Across Targets (Rule 22)**: Passing tests on one target or feature configuration does not establish repository standing. Different architectures compile differently. An implementation might achieve `CC=1` on x86_64 but contain hidden conditional jumps, panic paths, or loop backedges when compiled for another target. Architecture-specific instructions (like `PDEP`/`PEXT`) require separate, target-specific disassembly evidence.
3. **Source-level scanning is insufficient (Rule 20 & 7)**: The mandate states that "Source-level `CC=1` is necessary but insufficient." The ultimate truth is the compiled object code. If a target is skipped, the structural auditor (`@turing_machine`) cannot verify the absence of allocations, floating-point operations, or branches in that specific release object code.
4. **It forces the Integrity Score to 0 (Rule 24)**: A "gate-jurisdiction omission" is classified as an **absolute failure**. It cannot be masked by a weighted average; it immediately forces the Substrate Integrity Score (SIS) to `0` and triggers the strict `MaturityScrutiny` protocol. 

In short, reporting a passing scanner on an incomplete target matrix is considered a critical evasion (theater) because it violates the fundamental law that every byte of authoritative logic must be proven mathematically safe, branchless, and deterministic on *all* supported release targets.
