# Research Report: `ControlStateUnadmitted` Typed Refusal

## 1. Definition and Constitutional Mandate
Under **Rule 18 ("Typed refusals")** of the BCINR Deterministic Substrate Constitution (`AGENTS.md`), all rejected authoritative operations must produce a bounded typed refusal code. Human-readable error messages, panics, or language-level generic control flows (such as `Option::None` or unmapped `Result`) are explicitly prohibited in the execution hot path.

**`ControlStateUnadmitted`** is a mandated typed refusal category. It serves as the deterministic structural signal emitted when a proposed speculative candidate state fails to pass the admission pipeline (i.e., one or more validity predicates fail).

## 2. Branchless Mathematical Trigger Condition
The emission of `ControlStateUnadmitted` is tied directly to the system's branchless verification pipeline mandated by **Rule 10 ("No mutation before complete admission")** and **Rule 9 ("Mask-based execution law")**. 

When a candidate state ($x_{\mathrm{candidate}}$) is evaluated, a series of branchless validity predicates (e.g., bounds checking, cryptographic digest matching) produce a single unified boolean bitmask: $m_{\mathrm{admitted}}$.

The condition that triggers the refusal is when this mask evaluates to zero:
$$ m_{\mathrm{admitted}} = 0 $$

Because the architecture strictly prohibits control flow branches (`if invalid { return ... }`, $CC=1$), the system executes a bitwise polynomial masked commit that zeroes out the candidate state and instead restores the original immutable state ($x_t$):

$$ x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t) $$

Which structurally resolves via bitwise logic as:
$$ (m_{\mathrm{admitted}} \land x_{\mathrm{candidate}}) \lor (\neg m_{\mathrm{admitted}} \land x_t) $$

When $m_{\mathrm{admitted}} = 0$, the operation is rejected and the persistent state is left bit-for-bit unchanged. In lockstep with this operation, the runtime branchlessly selects and emits the `ControlStateUnadmitted` typed refusal in $O(1)$ constant time.

*(Note: Based on internal codebase documentation, the current implementation in `crates/bcinr-cmca/src/allocator.rs` calculates this mask correctly but currently returns `Option::None` instead of branchlessly emitting `ControlStateUnadmitted`, which is a recognized violation of Rule 18).*
