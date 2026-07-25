# Slow Rail in BCINR: Non-Authoritative Code

According to Rule 6 in the BCINR Deterministic Substrate Constitution (`AGENTS.md`), every source file and function must be classified into strict categories. The **Slow Rail** represents the non-authoritative boundary of the codebase.

## Definition

The Slow Rail is defined as the system's non-authoritative code that is exempt from the project's absolute runtime laws. While the authoritative runtime is strictly required to be bounded, branchless, and allocation-free, the slow rail is explicitly permitted to **branch and allocate memory** in order to handle complex, unbounded, or non-deterministic tasks.

## Permitted Operations

The Slow Rail handles higher-order tasks and heavy computation that cannot be constrained by the rigorous cyclomatic complexity (CC=1) laws of the deterministic substrate. It is permitted to perform operations including:

* **Data Processing & Parsing:** RDF parsing, SHACL validation, and artifact serialization.
* **Complex Computations:** Symbolic mathematics and eigenvalue search.
* **Derivation & Generation:** Certificate derivation and code generation.
* **User & Testing Interfaces:** CLI display, dashboards, test references, and benchmark orchestration.

Additionally, under Rule 12 (No runtime theorem discovery), the Slow Rail is responsible for deriving variables and theorems (e.g., spectral-radius estimation, adaptive threshold discovery, and optimization), such as $G$, $d$, $\delta$, $R_{\mathrm{noise}}$, and $R_{\mathrm{switch}}$.

## Strict Separation from the Authoritative Hot Path

The Constitution strictly dictates: **The slow rail may branch and allocate, but it must never be linked into or invoked from the authoritative hot path.** 

The reasons for this hard boundary are deeply embedded in the project's core architectural laws:
1. **Contamination of Determinism:** The authoritative runtime requires zero heap allocation, constant execution time, and no data-dependent branching. Allowing slow rail code into the hot path would violate these absolute rules by introducing unpredictable branches, dynamic iterations, and memory allocations.
2. **Clear Separation of Concerns (Discovery vs. Verification):** The slow rail is allowed to *discover* and *derive* theorems (e.g., power iteration, dynamic graph analysis), but the authoritative hot path is only allowed to *verify* a supplied, fixed witness through packed value comparison.
3. **Auditability and Enforcement:** Linking branching code into the hot path would cause the Enforcer of Determinism (`@turing_machine`) to fail the object-code and cyclomatic complexity audits, triggering a catastrophic failure of the Substrate Integrity Score (SIS).
