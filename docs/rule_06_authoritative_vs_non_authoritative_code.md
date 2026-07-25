# Rule 6: Authoritative versus Non-Authoritative Code

In the BCINR Deterministic Substrate, every source file and function must be strictly classified into one of four distinct categories. This classification dictates the architectural rules and constitutional laws that apply to the code.

## 1. The Authoritative Runtime
The authoritative runtime consists of any code capable of affecting core systemic state and execution guarantees. This includes code responsible for:
- Allocation
- Adaptive state and persistent state
- Admission and refusal masks
- Certificate verification
- Resource prices, semantic mass, and standing projections

**Rule Application:** This tier inherits *every absolute runtime law*. It must be mathematically bounded, strictly branchless (cyclomatic complexity $CC=1$), allocation-free, and entirely deterministic. 

## 2. The Slow Rail
The slow rail handles complex, variable-workload, or exploratory tasks that support the system but do not operate in the critical execution path. Responsibilities include:
- RDF parsing and SHACL validation
- Certificate derivation and symbolic mathematics
- Eigenvalue search
- Code generation and artifact serialization
- CLI display, dashboards, test references, and benchmark orchestration

**Rule Application:** The slow rail is permitted to branch, loop dynamically, and allocate memory. However, there is a strict isolation mandate: **slow rail code must never be linked into or invoked from the authoritative hot path**.

## 3. Test-Only Oracle
A test-only oracle is an independent mathematical specification used solely for verification and proof obligations.

**Rule Application:** Oracles must be explicitly excluded from production features. They serve as an independent source of truth to rigorously verify the authoritative runtime but never pollute the production runtime themselves.

## 4. Generated Authoritative Code
Generated authoritative code refers to source code that is generated programmatically but ultimately executed as part of the production runtime.

**Rule Application:** Generated code is not exempt from absolute runtime laws. After generation, it must pass all authoritative gates, including source scans, object-code audits, and branchless enforcement ($CC=1$).

---

## How This Separation Enforces Absolute Runtime Laws

The strict separation of these four tiers acts as an architectural firewall that makes the absolute runtime laws possible to enforce:

1. **Isolating Variable Complexity:** By explicitly pushing unbounded operations (like string parsing, optimization searches, and artifact serialization) to the **slow rail**, the **authoritative runtime** is freed from needing control-flow branches, allowing it to remain completely fixed-width, bounded, and allocation-free.
2. **Hot-Path Purity:** Because the slow rail can never be linked into or invoked from the hot path, data-dependent loops or dynamic dispatches cannot accidentally pollute the deterministic core.
3. **Uncompromised Verification:** The **test-only oracles** provide a pure mathematical reference to test the authoritative implementation against. This ensures that the highly-optimized, branchless arithmetic in the hot path mathematically aligns with the specification without accidentally deploying slow verification logic into production.
4. **Closing Generator Loopholes:** Subjecting **generated authoritative code** to the exact same rigorous object-code audits as hand-written code ensures code generation cannot be used as a backdoor to inject hidden branches, panics, or memory allocations into the runtime.
