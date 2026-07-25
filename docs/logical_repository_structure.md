# Logical Repository Structure: BCINR

The `bcinr` (BranchlessCInRust) repository is structurally organized around the stringent laws defined in its deterministic substrate constitution. This logical architecture physically separates bounded, branchless, authoritative execution from unconstrained, branching logic and enforces strict domain boundaries for verification, testing, and agent ownership.

## 1. Execution Domains: Hot Path vs. Slow Rail (Rule 6)

The codebase strictly segregates code into classified execution domains. The authoritative runtime (Hot Path) and the non-authoritative code (Slow Rail) are mutually isolated.

### Authoritative Runtime (The Hot Path)
The authoritative hot path is the bounded, allocation-free, branchless core. It must adhere to all absolute runtime laws (e.g., CC=1, zero heap allocation, no panic paths, no variable loops).
- **Scope**: Affects admission, adaptive state, resource prices, certificate verification, refusal masks, semantic mass, standing projections, and persistent state.
- **Constraints**: 
  - Strictly branchless (no `if`, `match`, data-dependent loops).
  - `#![no_std]` and zero heap allocation.
  - Mask-based execution and selection (e.g., `select(m, a, b)`).
  - No speculative mutation before complete admission.

### Non-Authoritative Code (The Slow Rail)
The slow rail consists of support and orchestration systems that are permitted to use standard language features but are categorically forbidden from being linked into or invoked by the hot path.
- **Scope**: RDF parsing, SHACL validation, certificate derivation, symbolic mathematics, eigenvalue search, code generation, artifact serialization, CLI display, dashboards, test references, and benchmark orchestration.
- **Constraints**: Allowed to branch and allocate memory, but inherently isolated from the authoritative runtime.

### Supplementary Domains
- **Test-Only Oracle**: Independent mathematical specifications that serve as reference semantics. Completely excluded from production features.
- **Generated Authoritative Code**: Generated code executed by the runtime. Must pass all stringent authoritative gates after generation.

## 2. Agent Isolation Domains (Rule 26)

To prevent self-certification and conflicts, the repository mandates strict separation of concerns and exclusive write ownership across distinct Agent Isolation Domains. Shared-file concurrent editing is strictly prohibited. 

| Domain | Exclusive Writer | Responsibilities |
| :--- | :--- | :--- |
| **Contracts and Proofs** | `@hoare_oracle` | Axiomatic proofs, specifications, Hoare contracts, valid domains, algebraic laws, and independent reference semantics. |
| **Scanners and Structural Gates** | `@turing_machine` | Structural audits, cyclomatic-complexity (CC=1) enforcement, authoritative call-graph classification, cheat-scanner policy, and disassembly evidence. |
| **Mutants and Hostile Fixtures** | `@armstrong_fault` | Adversarial test architecture, counterfactual mutant design, negative-domain testing, refusal-path verification, and test-suite adequacy. |
| **Authoritative Implementation** | `@von_neumann_bypass` | Branchless arithmetic design, SWAR construction, mask-based state selection, SIMD shuffles, and fixed-point mechanics. |

Other agents may review but cannot edit outside their domain without explicit ownership transfer in the work log. Generated files may only be written by an admitted generator.

## 3. The Gate Matrix (Rule 22)

A feature's correctness and standing are not established by passing tests in a single environment. The repository enforces a rigorous Gate Matrix that validates the code against all permutations of features, profiles, and architectures.

### Matrix Permutations
All verification gates must successfully run across every supported combination:
- `default features`
- `no default features`
- `all features`
- `release profile`
- `test profile` (where relevant)
- Supported architectures
- Generated clean tree

### Architecture-Specific Capabilities
For architecture-specific instructions (e.g., PDEP/PEXT), the matrix demands:
- An admitted target capability.
- A lawful fallback target or a bounded typed refusal.
- Separate disassembly evidence for the fallback, which must satisfy all identical structural laws (e.g., CC=1).

### Mandatory Verification Gates (Rule 23 integration)
To establish "PhD-Verified" standing (SIS = 100), the repository requires the execution and passing of specific admitted tasks before reporting results:
- `scan-cheats` (Anti-cheat scanner enforcing Rule 16/17)
- `contract-gate`
- `ci`
- `test-mutants` (Hostile mutation protocol)
- `audit-object-code` (Production-profile disassembly audit)
- `verify-generated` (Deterministic code generation)

Passing one feature configuration does not establish standing. Any gate failure, including absolute failures like a hidden branch or hot-path allocation, reduces the Substrate Integrity Score (SIS) to 0, blocks merges, and triggers `MaturityScrutiny`.
