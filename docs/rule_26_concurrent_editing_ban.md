Based on the `AGENTS.md` constitution, here is the explanation for Rule 26's restrictions on write ownership:

### Why "shared-file concurrent editing" is explicitly prohibited
Rule 26 enforces **exclusive write ownership** by mapping specific domains to strictly segregated agent roles (e.g., `@hoare_oracle` for proofs, `@von_neumann_bypass` for implementation). Shared-file concurrent editing is prohibited because it violates the foundational principles of the BCINR repository:

1. **Mandatory Independence (Rule 5 & Rule 27):** The constitution strictly prohibits self-certification. The agent writing the implementation cannot concurrently edit the mathematical proofs or hostile mutants. Shared editing would allow an agent to subtly alter a test or proof to accommodate a flawed implementation (e.g., a "silent repair"), destroying the adversarial and independent verification model.
2. **Clear Auditability and Accountability:** Every file must have a single, traceable "exclusive writer." Changes require "an explicit ownership transfer recorded in the work log" to ensure that the strict separation of powers (Mathematical law, Structural enforcement, Hostile verification, Implementation) is maintained and mathematically proven at every step.

### Why generated files may only be written by the admitted generator
This rule is a direct extension of **Rule 21 (Generated-code law)** and the project's absolute demand for determinism:

1. **Byte-Identical Reproducibility:** Generated code in BCINR must undergo a strict verification loop (`clean generation → digest output → regenerate → verify byte-identical output`). If any agent other than the admitted generator manually edits or modifies the file, it introduces "unexplained drift," which instantly invalidates the artifact's standing.
2. **Preserving the Authoritative Graph:** Generated code is executed by the runtime and is subject to the same strict laws as handwritten authoritative code (e.g., `CC=1`, object-code audits). If human or agent hands alter it, it bypasses the deterministic guarantee that a specific, mathematically-verified input shape will reliably produce a fixed output shape.
