# Slow Rail and Hot Path Boundary

According to the `bcinr` structural constitution, all code must be strictly classified to preserve the absolute runtime laws of the deterministic substrate. The ecosystem is broadly bifurcated into the **Authoritative Hot Path** and the **Slow Rail**. 

## The Slow Rail: Purpose and Permitted Relaxations

The authoritative hot path is bounded by extreme strictness: zero heap allocation, $CC=1$ (no branching), fixed bounded execution, and strict determinism. However, computing complex system properties, parsing dynamic formats, or generating mathematical proofs inherently requires unbounded loops, dynamic memory, and branch-heavy execution logic.

The **Slow Rail** exists to perform these unbounded, non-deterministic, or structurally complex computations. It is explicitly permitted to branch and allocate memory because it operates strictly *outside* the authoritative runtime boundary. Its role is to execute complex semantic derivations, theorems, and parsing logic offline or asynchronously, reducing all complex state down to static, verifiable structures (witnesses, certificates, and fixed values).

**Crucial Constraint:** While the slow rail can use branches and allocations, it **must never be linked into or invoked from the authoritative hot path.**

## Operations Strictly Confined to the Slow Rail

Any operation that implies runtime discovery, dynamic parsing, or allocation must be relegated to the slow rail. Per Rule 6 and Rule 12, the following operations are strictly confined to the slow rail:

* **Parsing & Validation:** RDF parsing, SHACL validation.
* **Mathematical Derivations:** Symbolic mathematics, Eigenvalue search, spectral-radius estimation, power iteration, Jacobian derivation, optimization over weighting vectors, Lyapunov search.
* **Theorem & Discovery:** Theorem discovery, adaptive threshold discovery, automatic $q$-range expansion, dynamic graph analysis, and Certificate derivation.
* **Engineering & CI Tasks:** Code generation, artifact serialization, CLI display, dashboards, test references, and benchmark orchestration.

## The Boundary: Handoff Mechanics to the Hot Path

Data handoff from the branching slow rail into the deterministic hot path must preserve the absolute runtime laws of the substrate. The transition of data relies on the architectural principle of **Verification over Discovery** and strict structural admission.

Data crosses the boundary safely via the following mechanisms:

1. **Decoupled Architecture:** The slow rail never directly calls the hot path. Data is handed off statically or sequentially as pre-computed payloads.
2. **Fixed-Size "Packed" Values:** The slow rail derives complex properties (e.g., deriving stability parameters such as $G, d, \delta, R_{noise}, R_{switch}$) and serializes them into strict, fixed-width configurations. The hot path only ingests these as "packed values."
3. **Certificates and Witnesses:** Instead of the hot path discovering a valid theorem (e.g., calculating an eigenvalue to ensure stability), the slow rail computes the mathematical proof and hands off a "fixed witness" or "certificate." The hot path simply executes branchless verification (e.g., verifying static domination $\widehat{G} \leq G_{certified}$).
4. **Structural Admission and Masking (Rule 10 & 11):** Once data reaches the hot path, it is subjected to complete admission without mutation. The hot path verifies all predicates in a branchless manner, resulting in an `AdmittedControlState` or `AcceptedCertificate`. The admission process uses bitwise boolean operations to derive an admission mask ($m \in \{0, 2^w-1\}$), followed by a fieldwise masked commit. The hot path relies on masked data selection (`select`) rather than control flow (`if/else`) to either ingest the slow rail's configuration or leave the state bit-for-bit unchanged. 
5. **Typed Refusals:** If the data handed off from the slow rail is invalid or the witness fails verification, the hot path will not branch to handle the error or panic. Instead, it rejects the state update structurally and yields bounded, typed refusals (e.g., `CertificateStale`, `DigestMismatch`, `ContractionMarginInsufficient`).
