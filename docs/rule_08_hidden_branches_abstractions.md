Here is the requested explanation regarding Rule 4 and Rule 8 from `AGENTS.md`:

### Branches Hidden in Trait Implementations
In Rust, traits are used to define shared interfaces, allowing functions to remain agnostic about concrete types. A function calling a trait method (e.g., `item.process()`) might look like a single, simple, branchless operation in the source code. However, the specific type implementing that trait could internally use `if`, `match`, loops, or early returns.

### Branches Hidden in Dependencies
Similarly, when your code calls a function from an external library or dependency, it might appear as a simple function call locally (e.g., `external_crate::compute(x)`). However, the internal logic of that external library could contain conditional branches or unbounded loops.

### Why They Violate the CC=1 Law
The core mandate of BCINR (as outlined in Rules 3, 4, 7, and 8) is a deterministic computational substrate where **"the authoritative instruction shape must not depend on semantic input."** 

This absolute requirement applies to the **entire transitive call graph**, not just the local function. According to Rule 7 (Whole-call-graph branchlessness), the final audited machine code must contain strictly zero input-dependent conditional jumps or loop backedges. 

Concealing branches behind abstraction boundaries—whether through trait polymorphism or external dependencies—violates the CC=1 law for the following reasons:
1. **Breaks Data Independence:** It introduces hidden data-dependent execution paths and potential timing side-channels, defeating the purpose of branchless bit-parallel arithmetic.
2. **Scanner Evasion:** It acts as a bypass to structural enforcement. Rule 8 explicitly states that private wrappers, macro-expansions, traits, and dependencies reachable from the authoritative call graph all count toward complexity. Hiding behavior behind abstractions is explicitly listed as a violation under Rule 16 (Anti-cheat manifesto: CHEAT-006).
3. **Fails the Object-Code Audit:** Even if the local Rust source looks like `CC=1`, the compiled object code will contain jumps linked from the trait or dependency, causing the mandate's final structural gate (Rule 20) to fail.

In short, the constitution demands byte-sequential control flow be entirely transformed into masking, arithmetic selection, and fixed lookup tables. Hiding branches simply moves the violation out of sight, which is strictly prohibited.
