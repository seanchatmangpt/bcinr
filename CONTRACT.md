# Formal Contracts — BranchlessCInRust

**Version:** 26.4.22  
**Status:** ALIVE  
**Verified:** 2026-07-25

## Overview

This document consolidates formal contracts for three critical subsystems: PDDL classical planning, POWL deterministic workflow compilation, and CMCA numeric stability algorithms. Each contract specifies:

1. **Domain** — The set of valid inputs and preconditions
2. **Invariants** — Properties that must hold throughout execution
3. **Refusal conditions** — When the system refuses to proceed and why
4. **Numeric laws** — Bounds, complexity, and resource guarantees
5. **State laws** — Correctness postconditions

## Contract Documents

### 1. PDDL Contract
**File:** `docs/contracts/PDDL_CONTRACT.md`

Formal specification of the PDDL 3.1 parser, grounder, and planner:
- Parser domain: PDDL 3.1 syntax per IPC 2014 specification
- Grounder invariants: variable substitution, effect composition
- Planner refusals: unsupported temporal conditions, negative goals
- Complexity: O(n³) ground state enumeration, O(2^n) worst-case plan length

### 2. POWL Contract
**File:** `docs/contracts/POWL_CONTRACT.md`

Specification of POWL workflow compilation to deterministic tapes:
- Domain: acyclic workflows with sequential and choice nodes
- Invariants: determinism (one execution path per state), temporal monotonicity
- Refusals: loops, data-dependent branching, unbounded concurrency
- Complexity: O(n) tape compilation, O(1) tape execution per tick

### 3. CMCA Contract
**File:** `docs/contracts/CMCA_CONTRACT.md`

Numeric stability contract for calibration and mass conservation:
- Domain: calibration inputs normalized to [0, 1]
- Invariants: mass conservation, allocation monotonicity
- Refusals: negative masses, division by zero, saturation bounds exceeded
- Stability: ±epsilon relative error, allocation = Σ(calibration)

## Hoare Logic Triples

**File:** `docs/contracts/HOARE_TRIPLES.md`

Formal Hoare logic verification of 15 theorems across the three subsystems:

| Theorem | Subsystem | Property |
|---------|-----------|----------|
| 1–6 | PDDL | Determinism, correctness, soundness, completeness |
| 7–9 | POWL | Preservation, compilation correctness, execution validity |
| 10–15 | CMCA | Stability, allocation determinism, mass conservation, branchlessness |

Each theorem includes:
- **Precondition** {P}
- **Command** C
- **Postcondition** {Q}
- **Proof reference** to thesis or test oracle

## Verification Status

✓ PDDL oracle: 4 conformance tests (IPC benchmarks)  
✓ POWL oracle: 5 deterministic scheduler tests  
✓ CMCA oracle: 1 baseline + 11 mutant tests  
✓ Hoare triples: All 15 theorems formally verified  

---

## See Also

- `docs/contracts/PDDL_CONTRACT.md` — PDDL domain specification
- `docs/contracts/POWL_CONTRACT.md` — POWL workflow semantics
- `docs/contracts/CMCA_CONTRACT.md` — CMCA numeric contract
- `docs/contracts/HOARE_TRIPLES.md` — Formal Hoare logic proofs
- `docs/gates/ORACLE_INDEPENDENCE.md` — Oracle verification architecture

**Last Updated:** 2026-07-25 | **Version:** 26.4.22
