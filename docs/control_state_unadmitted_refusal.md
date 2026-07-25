# Research Report: `ControlStateUnadmitted` Typed Refusal

## 1. Constitutional Mandate (Rule 18)
Under Rule 18 ("Typed refusals") of the BCINR Deterministic Substrate Constitution (`AGENTS.md`), all rejected authoritative operations must produce a bounded typed refusal code. The constitution explicitly prohibits human-readable text, panics, or language-level generic control flow (like `Option` or unmapped `Result`) in the hot path. `ControlStateUnadmitted` is mandated as one of these typed refusal categories to indicate that a proposed control state failed to pass the admission pipeline.

## 2. Relationship to Rule 10 ("No mutation before complete admission")
Rule 10 dictates that persistent state must never be mutated speculatively. Any update to the system state must follow a strict, branchless transaction shape:
1. Start with current immutable state ($x_t$)
2. Compute fixed-size candidate state ($x_{\mathrm{candidate}}$)
3. Verify all predicates
4. Derive a unified admission mask ($m_{\mathrm{admitted}}$)
5. Perform a fieldwise masked commit

The `ControlStateUnadmitted` refusal is the required structural signal to the caller when step 4 yields an admission mask of `0` (i.e., one or more predicates failed). Because Rule 10 prohibits early returns or branching logic before the commit (e.g., `if invalid { return Err(...); }`), this typed refusal must be selected and emitted branchlessly alongside the state commit operation.

## 3. Structural Trigger Circumstances
Structurally, the `ControlStateUnadmitted` refusal is triggered at the culmination of the candidate state verification pipeline. In practice, a series of validity predicates (such as bounds checking, cryptographic digest matching, and policy floors) are computed branchlessly to form a single combined boolean mask (`m_admitted`). 

When `m_admitted == 0`, the speculative state is deemed unadmitted. In this specific structural circumstance, the function is required to:
1. Execute a masked selection that discards the candidate state in favor of the existing state.
2. Branchlessly select and return the `ControlStateUnadmitted` typed refusal to inform the caller of the rejection. 

*(Note: Based on a codebase audit, the current implementation in `crates/bcinr-cmca/src/allocator.rs` calculates an `ok` mask correctly but violates Rule 18 and Rule 8 by returning `Option::None` instead of branchlessly emitting the `ControlStateUnadmitted` refusal.)*

## 4. Mathematical Enforcement of the Verification Pipeline
The requirement that unverified speculative candidate state cannot be persisted is enforced mathematically through bitwise polynomial arithmetic, entirely avoiding control flow branches (`CC=1`).

The lawful commit step takes the exact mathematical form mandated by Rule 10:
$$ x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t) $$

Per Rule 9 ("Mask-based execution law"), this selection is structurally equivalent to:
$$ (m_{\mathrm{admitted}} \land x_{\mathrm{candidate}}) \lor (\neg m_{\mathrm{admitted}} \land x_t) $$

By collapsing the entire verification pipeline into a full-width bitmask ($m_{\mathrm{admitted}} \in \{0, 2^w-1\}$), the architecture eliminates the possibility of partial state mutation or speculative execution. If any rigid verification predicate fails, the mask becomes completely zeroed. The bitwise logic then mathematically zeroes out the candidate state and fully restores the current immutable state ($x_t$). This guarantees that a rejected operation leaves the persistent state bit-for-bit unchanged in $O(1)$ constant time, completely independent of semantic input.
