# Substrate Integrity Score (SIS) and MaturityScrutiny Protocol

## Substrate Integrity Score (SIS)

The Substrate Integrity Score is a metric for evaluating the structural and mathematical correctness of the deterministic substrate. A component is only considered **"PhD-Verified"** if it scores **100/100** on the maturity matrix (Proof + Oracle + Hostile Tests).

The SIS is defined by the following formula:

$$ \text{SIS} = 100 - \sum_i w_i V_i $$

where $V_i$ are verified violations and $w_i > 0$ are their respective weights. 

### Absolute Failures

The following are absolute failures regardless of score. No weighted average may conceal a constitutional violation. Any of these failures immediately forces **`SIS = 0`** and triggers the **`MaturityScrutiny`** protocol:

* hidden authoritative branch;
* allocation in the hot path;
* unwitnessed mutation;
* surviving mutant;
* circular oracle;
* scanner evasion;
* stale certificate acceptance;
* state mutation after refusal;
* gate-jurisdiction omission;
* fabricated verification evidence.

---

## MaturityScrutiny Protocol

When `SIS < 100`, the `MaturityScrutiny` protocol is immediately enacted. Agents may not work around a failed gate by moving the feature elsewhere. The remediation requires following this strict 9-step process:

1. freeze feature development;
2. quarantine affected code;
3. identify all reachable authoritative symbols;
4. rerun proofs, scans, mutants, and disassembly;
5. produce a root-cause report;
6. repair the structural defect;
7. regenerate all dependent artifacts;
8. rerun the complete gate matrix;
9. issue a new standing receipt.
