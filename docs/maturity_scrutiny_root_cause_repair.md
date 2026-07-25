# MaturityScrutiny Protocol: Root-Cause Report and Structural Repair

Under the BCINR Deterministic Substrate Constitution (`AGENTS.md`), the **MaturityScrutiny protocol** (Rule 25) is the highest-severity remediation workflow. It is triggered whenever the Substrate Integrity Score (SIS) falls below 100, which includes any absolute constitutional failure (e.g., hidden authoritative branches, hot path allocation, surviving mutants, or fabricated evidence). 

This document outlines the specific requirements for producing the root-cause report, executing structural repairs, and explains the constitutional rationale for the mandatory regeneration of all standing artifacts.

## 1. Triggering `MaturityScrutiny`
According to **Rule 24**, any of the following absolute failures instantly force `SIS = 0` and trigger the protocol:
- Hidden authoritative branch or loop backedges
- Allocation in the hot path
- Unwitnessed mutation or state mutation after refusal
- Surviving mutant (failing the `@armstrong_fault` hostile mutation protocol)
- Circular oracle or scanner evasion
- Stale certificate acceptance
- Gate-jurisdiction omission or fabricated verification evidence

When triggered, development immediately freezes, affected code is quarantined, and all reachable authoritative symbols are identified (Steps 1-3).

## 2. Requirements for the Root-Cause Report (Step 5)
The root-cause report is a formal accounting of the constitutional breach. Based on repository laws, this report must satisfy:
- **Identification of the Breach**: It must pinpoint exactly which transcendent construct failed (e.g., mathematical contract, structural determinism, adversarial resistance).
- **Transitive Call Graph Impact (Rule 7)**: It must document how the defect bypassed existing architectural gates (e.g., did a branch hide in a macro expansion? Did a trait method introduce dynamic dispatch? Did a dependency introduce a panic path?).
- **Gate-Jurisdiction Omissions (Rule 16/23)**: It must explain *why* the automated `bcinr-cheat-scanner` or object-code audits failed to catch the violation (e.g., scanner evasion, benchmark theater, dead-path compliance).

> [!CAUTION]
> The root-cause report cannot rely on assertions like "appears safe" or "likely optimized." It must utilize the exact bounded standing claims and identify the precise mechanical failure in the verification matrix.

## 3. Requirements for Structural Repair (Step 6)
Repairing the defect is bound by strict execution laws and role isolations:
- **No Workarounds**: "Agents may not work around a failed gate by moving the feature elsewhere" (Rule 25). The defect must be met with a direct structural fix.
- **Strict Role Isolation (Rule 26 & 27)**: The fix requires independent orchestration. The implementation agent (`@von_neumann_bypass`) must rewrite the code using exclusively branchless logic (mask-based selection, SWAR, etc., per **Rule 9**). They *cannot* self-certify this fix. 
- **No Silent Repairs (Rule 5)**: A structural auditor (`@turing_machine`) may not silently repair code and then approve their own repair.
- **Whole-Call-Graph Compliance (Rule 7)**: The structural repair must guarantee that the *entire* transitive call graph (including compiler intrinsics and macro-generated branches) restores `CC=1` and absolute determinism. 

## 4. Why Complete Artifact Regeneration is Demanded (Steps 7-9)
The protocol strictly requires the regeneration of all dependent artifacts (Step 7) and rerunning the complete gate matrix (Step 8) before issuing a new standing receipt (Step 9). This is demanded for several core reasons:

### A. The "Load-Bearing Dependency" Principle
> *"Claims may not exceed their weakest load-bearing dependency."* (Rule 28)

Because BCINR is a deterministic substrate, its guarantees are holistic and transitive. A single byte change in an authoritative function cascades through the call graph, potentially altering object code, cyclomatic complexity, or memory layout. 

### B. Artifacts as the Sole Source of Standing
> *"Claims made outside these artifacts have no standing."* (Rule 29)

In BCINR, the runtime behavior is validated entirely by mechanistic, reproducible artifacts (e.g., `OBJECT_CODE_AUDIT.md`, `MUTANT_KILL_MATRIX.md`, `AUTHORITATIVE_CALL_GRAPH.md`). The system does not trust source code alone ("Source claims do not substitute for disassembly evidence" - Rule 4). Therefore, when the source is structurally repaired, the previous disassembly and verification artifacts instantly become stale and void. 

### C. Prevention of Self-Certification and Drift
> *"Generated files with unexplained drift invalidate standing."* (Rule 21)

The protocol forces a completely clean regeneration (`cargo make verify-generated`, clean-tree replay) to prevent "Agent agreement" or manual edits from slipping into the proof chain. Without full artifact regeneration, an agent might attempt to manually patch a single metric (like `CC=1`) while silently breaking an invariant or missing a new hidden branch in the object code. Complete regeneration acts as an unforgeable cryptographic seal that the new implementation satisfies all seven authoritative requirements.

> [!IMPORTANT]
> The requirement is absolute: **No checkpoint may be skipped because a later test passes** (Rule 30). The complete matrix—contracts, oracles, mutants, object-code audit, and source gates—must mechanically derive from the newly repaired state to yield a valid `ALIVE` or `BRANCHLESS_ALIVE` receipt.
