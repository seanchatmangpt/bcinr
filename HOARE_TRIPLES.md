# Hoare Logic Triples — BranchlessCInRust Subsystems

**This document is a reference pointer. See the authoritative version:**
**File:** `docs/contracts/HOARE_TRIPLES.md`

**Version:** 26.4.22  
**Scope:** Formal verification of PDDL, POWL, and CMCA subsystems  
**Method:** Hoare triples {P} C {Q}; wp(C, Q) weakest-precondition calculus

---

## Quick Reference: 15 Verified Theorems

| Theorem | Subsystem | Property | Status |
|---------|-----------|----------|--------|
| 1 | PDDL | Deterministic parsing | ✓ ALIVE |
| 2 | PDDL | Correct grounding | ✓ ALIVE |
| 3 | PDDL | Sound planning | ✓ ALIVE |
| 4 | PDDL | Complete planning | ✓ ALIVE |
| 5 | PDDL | Deterministic execution | ✓ ALIVE |
| 6 | PDDL | Plan receipt integrity | ✓ ALIVE |
| 7 | POWL | Preservation under compilation | ✓ ALIVE |
| 8 | POWL | Compilation correctness | ✓ ALIVE |
| 9 | POWL | Execution validity | ✓ ALIVE |
| 10 | CMCA | Numeric stability | ✓ ALIVE |
| 11 | CMCA | Allocation determinism | ✓ ALIVE |
| 12 | CMCA | Mass conservation | ✓ ALIVE |
| 13 | CMCA | Exploration floor | ✓ ALIVE |
| 14 | CMCA | Saturation safety | ✓ ALIVE |
| 15 | CMCA | Branchlessness | ✓ ALIVE |

---

## Proof Verification Status

- **All proofs:** Formal Hoare-logic notation in thesis and code comments
- **Oracle testing:** Each theorem has independent reference implementation + mutant tests
- **CI integration:** Automated test suites with 100% pass rate

---

## Full Specification

For complete formal definitions, preconditions, postconditions, and proof sketches, see:

**`docs/contracts/HOARE_TRIPLES.md`**

**Last Updated:** 2026-07-25 | **Version:** 26.4.22
