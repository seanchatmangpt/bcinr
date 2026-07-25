# Rule 25: MaturityScrutiny Protocol

In the BCINR Deterministic Substrate Constitution, the **MaturityScrutiny protocol** is the highest-severity remediation workflow. It is triggered immediately whenever the Substrate Integrity Score (SIS) falls below 100, which happens instantly upon any absolute constitutional failure (e.g., hidden authoritative branches, hot path allocation, surviving mutants, or fabricated evidence).

> [!WARNING]  
> When the protocol is triggered, **SIS = 0**. Agents may not work around a failed gate by moving the feature elsewhere. 

## The 9-Step Execution Pipeline

When `SIS < 100`, the system mandates a strict, unyielding 9-step sequence to restore integrity:

1. **Freeze feature development**
2. **Quarantine affected code**
3. **Identify all reachable authoritative symbols**
4. **Rerun proofs, scans, mutants, and disassembly**
5. **Produce a root-cause report**
6. **Repair the structural defect**
7. **Regenerate all dependent artifacts**
8. **Rerun the complete gate matrix**
9. **Issue a new standing receipt**

---

### Phase 1: Quarantine & Identification (Steps 1-3)
The immediate reaction to a constitutional breach is an absolute lockdown. 
- **Freeze Development:** All ongoing feature work is definitively halted.
- **Quarantine:** The compromised module or code path is completely isolated.
- **Symbol Tracing:** The system identifies *all* reachable authoritative symbols in the call graph to contain the blast radius, as deterministic guarantees are holistic and transitive.

### Phase 2: Root-Cause Report & Structural Repair (Steps 4-6)
Before any code is altered, a rigorous mechanical accounting of the failure is required.
- **Rerun Verification Matrix:** Prior artifacts are re-examined using proofs, scans, mutants, and object-code disassembly.
- **Produce Root-Cause Report:** 
  - Must pinpoint exactly which transcendent construct failed (e.g., mathematical contract, structural determinism, adversarial resistance).
  - Must map the transitive call graph impact (e.g., explaining how a branch hid in a macro expansion or trait method).
  - Must justify *why* the automated `bcinr-cheat-scanner` or object-code audits failed to catch the violation (e.g., scanner evasion, benchmark theater). Human assertions like "appears safe" are prohibited.
- **Structural Repair:** The defect must be met with a direct, branchless structural fix (mask-based selection, SWAR, etc.). The implementation agent cannot self-certify the fix, enforcing strict role isolation.

### Phase 3: Regeneration Pipeline (Steps 7-9)
Because BCINR is a hard deterministic substrate, its runtime behavior is validated entirely by mechanistic, reproducible artifacts (e.g., `OBJECT_CODE_AUDIT.md`), not just source code. 
- **Regenerate Artifacts:** Once the source is structurally repaired, previous disassembly and verification artifacts instantly become stale and void. The protocol forces a completely clean regeneration (`cargo make verify-generated`, clean-tree replay).
- **Rerun Complete Gate Matrix:** The complete matrix—contracts, oracles, mutants, object-code audit, and source gates—must mechanically derive from the newly repaired state.
- **Issue New Receipt:** Only after this exhaustive pipeline proves the new implementation satisfies all authoritative requirements can a new valid `ALIVE` or `BRANCHLESS_ALIVE` receipt be issued.
