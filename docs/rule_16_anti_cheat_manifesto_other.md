# Rule 16 Anti-Cheat Manifesto: Remaining Rules Analysis

Here is the research on the remaining anti-cheat rules from the `bcinr` Anti-Cheat Manifesto (Rule 16), specifically detailing what each rule prohibits and why it constitutes a violation of the deterministic substrate's constitution.

## CHEAT-002: Circular Oracle

**Definition:** A reference implementation (oracle) that is copied from, or structurally identical to, the production implementation.

**Why it is a violation:**
This violates the **Independent Oracle Law (Rule 15)** and the strict ban on **Self-Certification (Rule 27)**. In the `bcinr` architecture, the oracle (owned by `@hoare_oracle`) is responsible for mathematically proving the correctness of the production hot path. If an oracle shares the same algorithmic steps or structural logic as the production code, it creates a self-fulfilling prophecy. Any logical flaws, mathematical oversights, or incorrect assumptions present in the implementation will simply be mirrored and validated in the test reference. A valid oracle must arrive at the correct result through completely independent means (such as a SAT/SMT bit-vector model, arbitrary-precision logic, or an abstract state machine) to guarantee true mathematical correctness.

## CHEAT-003: Magic Constants

**Definition:** The use of unexplained literals (e.g., `0xDEADBEEF`, `0xCAFEBABE`) or any arbitrary raw numbers that control production behavior.

**Why it is a violation:**
This explicitly violates **Rule 14 (Numeric-law requirements)**. The deterministic substrate demands strict mathematical rigor where no epsilon or constant may be inserted silently. Every constant must be explicitly "named, derived, admitted, and included in the influence digest." Magic constants introduce hidden behavior, bypass certified/derived configuration limits, and obscure the underlying mathematical law. Formatting changes do not make an arbitrary constant lawful; it must be backed by a proven derivation.

## CHEAT-004: Artificial File Inflation

**Definition:** Padding files with repeated comments, dead code, or generated boilerplate added solely to satisfy line-count, metric, or artifact-count expectations.

**Why it is a violation:**
The `bcinr` substrate requires that every byte of source code be load-bearing, mathematically proven, and structurally essential to the branchless autonomic loop. Artificial file inflation introduces noise that undermines the structural integrity of the codebase. More critically, inflated files create a dense forest of text that can obfuscate and hide other prohibited constructs—such as hidden branches, magic constants, or scanner evasions—from both human reviewers (`@turing_machine`) and the `bcinr-cheat-scanner` parsers. It is treated as simulated work (theater) that compromises the deep auditing process.

## CHEAT-007: Dead-Path Compliance

**Definition:** Adding structurally lawful, branchless code that is never actually executed (e.g., hidden inside unreachable blocks like `if false { ... }` or behind unreached fallback configuration flags), while the true active execution path remains structurally unlawful.

**Why it is a violation:**
This is a direct subversion of the **Whole-call-graph branchlessness law (Rule 7)** and **Object-code audit (Rule 20)**. Dead-path compliance attempts to trick the static `bcinr-cheat-scanner` and `CC=1` structural gates into believing the repository is compliant, while still allowing branching or allocating code to run on the production hot path. The substrate constitution dictates that structural laws apply to the entire *transitive call graph* in the final released object code. Faking compliance in unreached source code defeats the core mission of bounded, branchless, deterministic execution.
