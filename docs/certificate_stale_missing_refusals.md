# `CertificateStale` and `CertificateMissing` Refusals in BCINR

In the BCINR deterministic substrate, **Typed Refusals** (mandated by Rule 18 in `AGENTS.md`) are the standard mechanism for rejecting authoritative operations. Because the runtime is strictly branchless and forbids runtime theorem discovery, pre-computed cryptographic certificates act as proofs of stability (e.g., derived by the branching "slow rail"). `CertificateStale` and `CertificateMissing` are the specific typed refusals raised when these proofs are invalid.

## 1. Specific Circumstances for the Refusals

*   **`CertificateMissing`**: This refusal is raised when an authoritative operation requiring a verified state context—such as adaptive learning or mode switching—is invoked without any certificate being provided. It indicates a domain condition where the upstream caller fundamentally failed to supply or seal a certificate receipt (e.g., bypassing the required `CertificateReceipt` input). 
*   **`CertificateStale`**: This refusal is raised when a certificate is presented, but it is no longer valid for the current system state. The total digest ($H_a$) must perfectly bind to all generated tables, kernel implementations, bounds, comparison matrices, and switching laws. If *any* of these parameters drift or change upstream, or if a structural certificate mismatch occurs during a control mode transition, the digest verification fails and `CertificateStale` is triggered. 

In both circumstances, the loss of a stability certificate transitions the runtime out of `CertifiedLearning` and into a fallback mode (e.g., `CertifiedSelectionOnly` / `CMCA_LEARNING_FROZEN`), where adaptive state mutation is frozen, but deterministic selection continues.

## 2. Branchless Refusal Mechanics

Under the **Radon Law ($CC=1$)**, the runtime is strictly prohibited from using data-dependent branches (`if`, `match`, early returns, or panics) to handle absent or outdated certificates. Operations are refused structurally through **Masked State Commitment**:

1.  **Branchless Error Accumulation**: 
    The conditions representing "missing" or "stale/mismatched digest" are evaluated into numeric values (`1` for true, `0` for false). These boolean values are passed into constant-time bitwise functions (like `masked()` and `union()`) to accumulate refusal bitflags directly into a numeric `RefusalSet` mask, bypassing control flow entirely.

2.  **No Speculative Mutation**: 
    Per Rule 10, persistent state is never mutated speculatively. The runtime calculates the entire candidate state in fixed-size stack scratch space, parallel to verifying all required predicates (including the cryptographic certificate).

3.  **Masked Selection**: 
    Instead of branching (`if valid { commit } else { reject }`), the runtime derives a full-width admission mask ($m \in \{0, 2^w-1\}$) from the `RefusalSet`. It then performs a fixed structural selection:
    `next_state = State::select(admission_mask, candidate_state, current_state)`

4.  **Bit-for-Bit Preservation**: 
    If a `CertificateMissing` or `CertificateStale` refusal is accumulated, the admission mask ensures that the selection function deterministically picks the `current_state` over the `candidate_state`. The operation is structurally rejected, and the persistent state remains bit-for-bit unchanged, fully satisfying the branchless contract.
