Based on the `AGENTS.md` constitution, here is the detailed classification and rules for the **Slow rail** under Rule 6 (and additionally referenced in Rule 12):

### Operations Classified as Slow Rail
Code is designated as "Slow rail" if it performs any of the following tasks:
* RDF parsing
* SHACL validation
* Certificate derivation
* Symbolic mathematics
* Eigenvalue search
* Code generation
* Artifact serialization
* CLI display
* Dashboards
* Test references
* Benchmark orchestration

Additionally, per **Rule 12 (No runtime theorem discovery)**, the slow rail is responsible for all theorem discovery and derivation. Specifically, it derives stability witnesses and parameters such as:
* $G,\ d,\ \delta,\ R_{\mathrm{noise}},\ R_{\mathrm{switch}}$

### What is Permitted
Unlike the authoritative runtime (which is bound by strict deterministic laws), code in the slow rail is explicitly permitted to:
* **Branch:** Use control flow, conditional logic, and data-dependent loops.
* **Allocate:** Perform heap memory allocations.

### What is Strictly Prohibited
* **Linking & Invocation:** The slow rail **must never be linked into or invoked from the authoritative hot path**. The hot path must maintain absolute isolation to guarantee its zero-allocation, branchless (`CC=1`) execution. For example, while the slow rail discovers and derives theorems, the hot path may only verify or compare the packed values.
