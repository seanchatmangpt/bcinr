# Rule 28: Standing Vocabulary

According to Rule 28 in `AGENTS.md`, the BCINR substrate mandates the use of bounded, precise standing labels to communicate the verification status of code and theorems.

## Allowed Standing Labels

The authoritative allowed standing labels and their definitions are:

* **`PROVEN`**: A specific theorem is machine-checked or exhaustively established over its declared domain.
* **`INVARIANT`**: True by construction or type exclusion.
* **`ALIVE`**: The implementation executes and passes all declared gates in the pinned environment.
* **`SOURCE_BRANCHLESS_PARTIAL`**: Source appears branchless, but complete object-code standing is not established.
* **`BRANCHLESS_ALIVE`**: The authoritative call graph passes source, complexity, allocation, panic, and disassembly audits.
* **`REPORTED_ALIVE`**: An agent reports success, but independent reproduction has not occurred.
* **`PARTIAL_ALIVE`**: Some required gates remain incomplete.
* **`UNKNOWN`**: Evidence is insufficient.
* **`REFUSED`**: The input or configuration is outside the admitted domain.
* **`BUILD_BROKEN`**: The pinned build fails.

## Weakest Dependency Rule

Claims regarding standing may not exceed their weakest load-bearing dependency. If an authoritative function depends on a helper with `PARTIAL_ALIVE` standing, the parent function cannot claim `BRANCHLESS_ALIVE` even if it is structurally perfect itself.

## Other Key Status Flags

* **`MUTATION_GATE_FAILED`** (from Rule 19): If a hostile mutant survives testing, it changes the project standing to this status, immediately blocking all feature work.
* **`REPORTED`**: Used when an agent reports a finding without reproducing it locally. Agent agreement is not evidence; it must transition from `REPORTED` to `ALIVE` via a machine-checked artifact.
