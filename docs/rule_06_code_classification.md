# Rule 6: Code Classification (Authoritative vs. Non-Authoritative)

In the `bcinr` deterministic substrate, every source file and function must be strictly classified to enforce the project's zero-allocation, branchless guarantees. The system bifurcates operations into four distinct categories:

## 1. Authoritative Runtime (The Hot Path)
The authoritative runtime is the core execution engine of the substrate. 

**Scope:**
This classification applies to any code that can affect:
* Allocation
* Adaptive state
* Admission
* Certificate verification
* Refusal masks
* Resource prices
* Semantic mass
* Standing projections
* Persistent state

**Constraints:**
Code in the authoritative runtime inherits **every absolute runtime law**. It must be strictly branchless (CC=1), perform zero heap allocations, have no data-dependent loops, no panic paths, no unwinding, and operate entirely with deterministic, bitwise polynomials.

## 2. Slow Rail
The slow rail is designed for complex, non-deterministic, or resource-intensive tasks that occur outside the critical execution path.

**Scope:**
This encompasses code performing:
* RDF parsing and SHACL validation
* Certificate derivation
* Symbolic mathematics and eigenvalue search
* Code generation
* Artifact serialization
* CLI display and dashboards
* Test references and benchmark orchestration

**Constraints:**
Unlike the authoritative runtime, the slow rail **may branch and allocate memory**. However, the strict boundary must be preserved: slow rail code must **never** be linked into or invoked from the authoritative hot path.

## 3. Test-Only Oracle
An oracle serves as the ground-truth mathematical specification against which the authoritative runtime is verified.

**Scope:**
It is an independent mathematical specification completely excluded from production features. 

**Constraints (per Rule 15):**
Oracles must be **structurally and logically distinct** from production code. They cannot be simple line-by-line translations or reuse production lookup tables/helpers. Permitted forms include direct mathematical formulas, abstract state machines, arbitrary-precision implementations, or SAT/SMT bit-vector models. They must be reviewed by the axiomatic proof lead (`@hoare_oracle`), not the implementation owner.

## 4. Generated Authoritative Code
This refers to source code that is programmatically generated (often by the slow rail) but is executed by the runtime hot path.

**Scope:**
Generated source that executes within the authoritative runtime boundary.

**Constraints (per Rule 21):**
Generated code is **not exempt** from the runtime laws. Once generated, it must:
* Pass all authoritative gates (CC=1, cheat scanner, object-code disassembly inspection).
* Be entirely reproducible (byte-identical upon regeneration).
* Contain no hidden branches or fixture-specific identifiers.
* Bind to the source graph and certificate digests.
* Hand-editing generated output is strictly prohibited.
