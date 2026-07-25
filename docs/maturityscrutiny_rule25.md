# Rule 25: MaturityScrutiny Protocol Enforcement

### 1. Enforcing Freeze and Quarantine (`SIS < 100`)
When the Substrate Integrity Score (SIS) drops below 100 (often forced to `0` due to absolute constitutional failures like hidden branches, allocations, or surviving mutants), the **MaturityScrutiny protocol** is automatically triggered, requiring a strict 9-step remediation process.

*   **Freeze Feature Development (Step 1):** All ongoing product or feature work is immediately and definitively halted across the tree. The repository mandates exclusive domain ownership during this state, granting `@turing_machine` (structural enforcer) exclusive write access. The absolute priority is restoring the deterministic integrity of the substrate; developers are strictly forbidden from evading the failure by moving the feature elsewhere.
*   **Quarantine Affected Code (Step 2):** Unverified or structurally non-compliant logic is physically isolated (e.g., moved to a specific `quarantine/` directory boundary). This guarantees the defective code cannot pollute the authoritative hot path or be executed by the primary gates. CI tasks are re-engineered to verify artifacts strictly without invoking the quarantined code until the structural defect is properly repaired via mathematical constructs.

### 2. The Standing Receipt Format
After the defect is repaired, all dependent artifacts are regenerated (Step 7), and the complete CI gate matrix is rerun (Step 8), a **new standing receipt** must be issued (Step 9) to formally restore `SIS = 100` and resume feature development. 

According to **Rule 23 (Required repository gates)**, the required format for reporting the result of these gates (the standing receipt) must state exactly:

```text
command
exit status
files inspected
features inspected
targets inspected
findings
artifact digest
```

This verified artifact digest acts as reproducible mechanical evidence proving that all tasks had full jurisdiction over the changed files, and the repaired codebase deterministically compiles into a fully compliant, branchless state.
