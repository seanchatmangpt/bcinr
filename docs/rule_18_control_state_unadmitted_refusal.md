Here is the research report detailing the `ControlStateUnadmitted` typed refusal and its role in the BCINR runtime, based on the constitution (`AGENTS.md`) and internal documentation.

# Research Report: `ControlStateUnadmitted` Typed Refusal

## 1. What it is
Under **Rule 18 ("Typed refusals")** of the BCINR Deterministic Substrate Constitution, all rejected authoritative operations must produce a bounded typed refusal code. Human-readable text, panics, and language-level generic control flow (like `Option` or unmapped `Result`) are explicitly prohibited in the hot path. 

**`ControlStateUnadmitted`** is one of these mandated typed refusal categories. It serves as the deterministic failure signal emitted when a proposed speculative candidate state fails to pass the admission pipeline (i.e., one or more validity predicates fail).

## 2. Role in the Runtime
Its role is deeply intertwined with **Rule 10 ("No mutation before complete admission")** and **Rule 9 ("Mask-based execution law")**, which require all state mutations to follow a strict, branchless transaction shape.

1. **The Verification Pipeline**: When the system proposes a speculative candidate state ($x_{\mathrm{candidate}}$), it evaluates a series of validity predicates (e.g., bounds checking, cryptographic digest matching) completely branchlessly. This yields a single combined boolean mask, $m_{\mathrm{admitted}}$.
2. **Branchless Rejection**: If any check fails, the mask evaluates to `0`. Because branching (e.g., `if invalid { return Err(...); }`) is strictly prohibited ($CC=1$), the runtime must branchlessly execute a masked selection that discards the candidate state in favor of the existing state ($x_t$).
3. **Emitting the Refusal**: In tandem with zeroing out the candidate state, the runtime branchlessly selects and returns the `ControlStateUnadmitted` typed refusal to inform the caller of the rejection.

Mathematically, the state commit takes the exact form:
$$ x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t) $$

If $m_{\mathrm{admitted}} == 0$, the speculative state is not admitted. The state remains bit-for-bit unchanged, and `ControlStateUnadmitted` is returned in $O(1)$ constant time, preserving the axiomatic calculus of the branchless substrate.

*(Note: In contrast, a successful validation where $m_{\mathrm{admitted}} \neq 0$ forms an `AdmittedControlState` proof under Rule 11, authorizing the transition).*
