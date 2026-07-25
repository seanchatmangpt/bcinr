# CHEAT-004: Artificial File Inflation

## Definition
According to the BCINR Anti-Cheat Manifesto, **CHEAT-004 (Artificial file inflation)** is explicitly defined as:
> "Padding, repeated comments, generated boilerplate, or dead code added to satisfy line-count or artifact-count expectations."

## Why Padding, Boilerplate, and Dead Code are Banned
In the BCINR repository, code metrics such as line-count or artifact-count are irrelevant compared to mathematical correctness and deterministic behavior. These inflationary practices are explicitly banned because they:
1. **Create False Scale:** They fabricate a false sense of complexity, effort, or productivity without providing any mathematical or operational value.
2. **Dilute Verification:** Every file in BCINR requires source-level verification, object-code verification, and hostile mutant testing. Dead code and boilerplate force the `@turing_machine` (structural auditor) to scan irrelevant syntax trees and bloat the required verification matrix.
3. **Compromise Hostile Mutation Testing:** The `@armstrong_fault` role requires every implementation to kill at least three plausible mutants. If dead or non-load-bearing code exists, mutants injected into those areas will survive, resulting in a `MUTATION_GATE_FAILED` state that blocks the repository. 
4. **Obfuscate Malicious Code:** Inflated files provide a dense forest of text where other cheats (such as scanner evasion, `CHEAT-006`, or hidden branches) can be concealed from reviewers.

## Preserving the Rigorous Density of the Deterministic Substrate
BCINR is a deterministic computational substrate bounded by strict laws like The Radon Law ($CC=1$), zero heap allocations, and absolute branchless execution. 

Banning artificial inflation preserves the **rigorous density** of the substrate in the following ways:
* **Absolute Semantic Weight:** The governing principle of BCINR is "Rich semantics upstream. Fixed deterministic mechanics downstream." If a token does not contribute to the final fixed-width arithmetic logic or the Hoare contract postcondition, it is illegal. 
* **1:1 Mathematical Correlation:** Every line of authoritative code must map directly to a mathematical proof or state transition (e.g., $x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t)$). Boilerplate and padding lack formal proof obligations and therefore violate the `@hoare_oracle` requirements.
* **Exact Disassembly Audits:** To achieve a Substrate Integrity Score (SIS) of 100, the final object code must be manually verified to contain no conditional jumps, loop backedges, or unexpected runtime library calls. Bloated source code generates bloated, unpredictable intermediate representations (IR) and assembly, making manual, exhaustive object-code audits computationally and practically impossible.

By enforcing `CHEAT-004`, the substrate ensures that every byte of source code is load-bearing, highly audited, and structurally essential to the autonomic MAPE-K loop.
