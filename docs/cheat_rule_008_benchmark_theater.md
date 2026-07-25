### CHEAT-008: Benchmark Theater

According to **Rule 16 (Anti-cheat manifesto)**, CHEAT-008 is defined as:
> *Benchmarking a stub, constant-folded path, dead result, or reduced problem not equivalent to production.*

#### Why it is a Constitutional Violation

Benchmarking simplified or dead code is strictly prohibited in the BCINR substrate for the following constitutional reasons:

1. **Fabrication of Verification Evidence is an Absolute Failure**
   Under **Rule 24 (Substrate Integrity Score)**, producing "fabricated verification evidence" is listed as an absolute failure. It bypasses the requirement that every primitive must provide an independent oracle, hostiles mutants, and reproducible evidence. An absolute failure overrides any weighted average, forces the project's Substrate Integrity Score (SIS) to **0**, and triggers the strict `MaturityScrutiny` protocol (freezing feature development and quarantining code).

2. **Performance Claims Cannot Override the Constitution**
   **Rule 2 (Constitutional Precedence)** explicitly states that *"Claims such as 'faster', 'simpler', 'idiomatic', or 'the compiler will optimize it' do not override this constitution."* A benchmark that relies on a reduced problem or compiler-optimized constant-folded path artificially props up performance claims without executing the required, mathematically rigorous branchless logic. 

3. **Strict Evaluation of Equivalent Production State**
   BCINR demands exact deterministic execution where the "authoritative instruction shape must not depend on semantic input" (Rule 4). Benchmarking a stub or a problem not equivalent to production tests a different graph of instructions than what runs in the hot path, violating the core mandate that the authoritative execution must be verified exactly as it behaves in production. 

4. **Mandatory Scanner Enforcement**
   To actively prevent Benchmark Theater, **Rule 17 (Cheat-scanner requirements)** mandates that the `bcinr-cheat-scanner` must explicitly *inspect benchmark targets* to ensure they are fully lawful and executing production-equivalent operations. Any cheat finding instantly blocks merges.
