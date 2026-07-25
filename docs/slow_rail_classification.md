# Rule 6: Authoritative vs. Non-Authoritative Code (The "Slow Rail")

In the BCINR deterministic substrate, Rule 6 of the `AGENTS.md` constitution strictly partitions all codebase operations into two mutually exclusive domains: the **Authoritative Runtime** (the hot path) and the **Slow Rail** (non-authoritative code).

This classification ensures that the core runtime remains a mathematically verifiable, branchless, and allocation-free environment while still supporting complex, real-world data processing like RDF parsing and SHACL validation.

## 1. What is the Slow Rail?
The **Slow Rail** encompasses all orchestrational, non-deterministic, or variable-workload support systems that surround the runtime. According to Rule 6, this explicitly includes code performing:
- **Data Parsing & Semantic Validation:** RDF parsing, SHACL validation, and artifact serialization.
- **Theorem Discovery & Cryptography:** Certificate derivation, symbolic mathematics, eigenvalue search.
- **Tooling:** Code generation, CLI display, dashboards, test references (oracles), and benchmarking.

Because these tasks fundamentally require variable-length graph traversal, dynamic memory heap allocations (`Vec`, `String`), and data-dependent control flow (`if`, `while`, `match`), they are granted explicit exemption from the hot path's stringent structural rules. The Slow Rail is freely permitted to branch and allocate.

## 2. Structural Classification & Enforcement
To prevent the Slow Rail from compromising the Authoritative Runtime's absolute laws (e.g., `#![no_std]`, `CC=1`, `zero heap allocation`), the project structurally classifies and isolates code using the following mechanisms:

### Strict Call Graph Isolation (No Linkage)
Rule 6 dictates that the Slow Rail **"must never be linked into or invoked from the authoritative hot path."** 
Because the project's absolute runtime laws apply *transitively*, a branchless public function calling a branching parsing helper is a constitutional violation. Thus, RDF parsing and SHACL validation crates or modules are physically separated at the `Cargo` / crate dependency level. The hot path binary cannot invoke or link against them. 

### Separation of Derivation vs. Verification
The structural barrier between the two rails heavily relies on Rule 12 ("No runtime theorem discovery"). 
Instead of the runtime traversing and validating complex SHACL structures dynamically, the system relies on an offline/asynchronous hand-off:
1. **Derivation (Slow Rail)**: The Slow Rail parses the RDF, validates SHACL logic, derives required stability certificates, and computes matrices ($G, d, \delta$). 
2. **Generation**: The Slow Rail acts as a compiler, outputting *Generated Authoritative Code* (e.g., `cmca_generated.rs`, `generated/case_studies.rs`).
3. **Verification (Hot Path)**: The hot path executes without discovery logic, merely verifying the static pre-computed artifacts against deterministic bounds (e.g., checking if $Gd \le (1-\delta)d$) using fixed-width, bit-parallel masking arithmetic.

### Gate Matrix Scanners & Audits
The classification is structurally enforced by CI verification tooling (the `bcinr-cheat-scanner`, `audit-object-code`, and the `@turing_machine` role constraints).
These tools disassemble the production release object code of the hot path to ensure no panic paths, allocator symbols, floating-point instructions, or unexpected conditional jumps have slipped through. If Slow Rail code (like RDF parsing loops) accidentally enters the hot-path transitive graph, the object-code audit automatically fails and lowers the Substrate Integrity Score (SIS) to 0, blocking the build.
