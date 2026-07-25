# ContractViolation and UnsupportedDomain Refusals

In the BCINR deterministic substrate, **Rule 18 (Typed Refusals)** of the constitution (`AGENTS.md`) mandates that all rejected authoritative operations must produce bounded, typed refusal codes. Human-readable text is strictly banned from the hot path. Among the mandated refusal categories, `ContractViolation` and `UnsupportedDomain` are mathematically central to the integrity of the system.

## Definitions

*   **`ContractViolation`**: Emitted when an input or state breaches the strict Hoare logic contracts ($\{P(x)\} \quad f(x) \quad \{Q(x,f(x))\}$) established by the `@hoare_oracle`. This indicates a failure of axiomatic constraints, such as conservation laws, monotonicity, or specific state-mutation boundaries.
*   **`UnsupportedDomain`**: Emitted when an input falls outside the exact bounds of the mathematically admissible domain explicitly verified and proven for a given primitive.

## Why Silent Mitigations Are Strictly Banned

Rule 18 explicitly prohibits operations from silently clamping outside admitted policy, dropping a factor, falling back to a simpler algorithm, mutating partial state, or returning a plausible default when encountering unsupported inputs. The substrate enforces returning typed refusals instead for the following architectural reasons:

### 1. The Radon Law ($CC=1$) and Branchless Instruction Shape
According to **Rule 8 (Absolute CC=1 Law)**, any data-dependent branch (`if`, `match`, early returns) is illegal in the authoritative call graph. "Falling back to a simpler algorithm" or "conditionally dropping a factor" inherently requires control-flow branching based on the input's semantic value (e.g., `if unsupported { fallback_algorithm() } else { normal_algorithm() }`). This violates the core tenet that the runtime machine code instruction shape must remain fixed, predictable, and identical regardless of the input data.

### 2. Rule 10: No Mutation Before Complete Admission
The substrate demands that a rejected operation leaves persistent state bit-for-bit unchanged. Attempting to "fix up" an unsupported input by returning a plausible default or silently clamping violates the transactional boundary of the system. State transitions must occur via a branchless masked commit (`select(admission_mask, candidate, current)`). If the input violates the allowed domain, the admission mask evaluates to `0`, discarding the candidate state entirely rather than speculatively polluting it with a partial or "clamped" mitigation.

### 3. Destruction of Axiomatic Proofs and Determinism
Every authoritative primitive in BCINR requires full-domain standing—an independent mathematical oracle or proof that the fixed-width arithmetic is correct across the *entire* admitted domain. 
*   If the system silently drops a factor or clamps an unsupported input, it alters the mathematical conservation laws without proof.
*   The output no longer corresponds to the rigorous object-code and Hoare logic contracts audited by the `@turing_machine` and `@hoare_oracle`.
*   Silent alterations destroy the predictability and provability of the runtime, directly conflicting with the core mission: $\text{admitted input} \rightarrow \text{fixed instruction shape} \rightarrow \text{deterministic output}$.

### 4. Rule 12: No Runtime Theorem Discovery
The authoritative runtime is designed to mechanically verify static constraints, not to discover stability on the fly. Dynamically adapting to an unsupported domain by defaulting to a simpler algorithm constitutes a "runtime algorithm search" or "adaptive threshold discovery," which are expressly forbidden. The substrate must evaluate the input exactly as requested; if the fixed domain bounds are violated, it must halt the transition completely rather than improvising.

## Conclusion

Returning `ContractViolation` or `UnsupportedDomain` as a typed refusal allows the substrate to bubble up the exact mathematical boundary failure using branchless bitwise intersections. This guarantees the refusal reaches the top boundary of the authoritative runtime without executing a single conditional instruction, allocating memory, or speculatively altering state. Silent clamping and fallbacks are illusions of resilience that hide complexity; BCINR requires the structural honesty of a complete, branchless rejection.
