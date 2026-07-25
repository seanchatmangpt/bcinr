# Authoritative versus Non-Authoritative Code

In the BCINR Deterministic Substrate, every source file and function must be strictly classified into specific execution tiers. Rule 6 of the `AGENTS.md` constitution defines a hard boundary between these tiers to preserve the mathematical guarantees of the runtime.

## The Boundary Classifications

### 1. Authoritative Runtime (The Hot Path)
The authoritative runtime is the deterministic core of the system. It includes any code that can affect:
- Allocation
- Adaptive state
- Admission
- Certificate verification
- Refusal masks
- Resource prices
- Semantic mass
- Standing projections
- Persistent state

**Boundary Rules:** This code inherits *every absolute runtime law* defined in the constitution (Rule 3). It must be allocation-free (zero heap allocation), branchless (Cyclomatic Complexity = 1), panic-free, unwinding-free, and execute in bounded fixed time without dynamic dispatch or floating-point operations. These laws apply transitively to the entire call graph.

*(Note: Generated code executed by the runtime falls under "Generated authoritative code" and is treated exactly like the authoritative runtime. It is not exempt and must pass all authoritative gates, audits, and checks after generation.)*

### 2. Slow Rail
The slow rail handles the complex, non-deterministic, or variable-workload operations that support the system but do not require strict execution bounds. It includes code performing:
- RDF parsing and SHACL validation
- Certificate derivation
- Symbolic mathematics and eigenvalue search
- Code generation
- Artifact serialization
- CLI display and dashboards
- Test references and benchmark orchestration

**Boundary Rules:** Unlike the authoritative runtime, the slow rail is explicitly permitted to branch, allocate memory, and perform variable graph traversals.

### 3. Test-Only Oracles
An independent mathematical specification that is explicitly excluded from production features. 
**Boundary Rules:** Oracles must be structurally and logically distinct from the production implementation. They provide an independent reference (e.g., direct mathematical formula, abstract state machine, SAT/SMT bit-vector model) for verifying the deterministic output of the authoritative runtime.

---

## Why the Slow Rail Must Never Touch the Hot Path

The slow rail must **never be linked into or invoked from the authoritative hot path** due to the transitive nature of BCINR's Absolute Runtime Laws.

1. **Transitive Contamination:** The constitution dictates that absolute runtime laws (like `CC=1`, zero allocations, no panics, no branches) apply *transitively* (Rules 3 and 7). If an authoritative, branchless function calls a private helper or an external module that branches or allocates, the entire call graph is invalidated.
2. **Determinism and Execution Bounds:** The slow rail performs operations like runtime parsing (RDF/SHACL), unbounded iterations (eigenvalue searches), and dynamic memory allocation. Allowing these operations to be reachable from the hot path would destroy the "fixed bounded execution work" and "fixed bounded memory access" guarantees.
3. **Branchless Arithmetic Requirements:** The authoritative runtime is built on bit-parallel mechanics over byte-sequential control flow. Injecting the slow rail's branching operations (like standard `if` statements or parsing loops) would violate the masking and selection semantics (Rule 9) required to prevent timing side-channels and guarantee deterministic output for every instruction.

In short, linking the slow rail into the hot path compromises the architectural integrity of the deterministic substrate, instantly failing structural audits (`@turing_machine`) and invalidating the whole-call-graph branchlessness guarantee.
