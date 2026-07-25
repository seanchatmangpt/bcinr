Here is the detailed research based on `AGENTS.md` and the internal `bcinr` substrate documentation regarding the `ReceiptRejected` typed refusal and its relationship to the `ReceiptSound` law.

### The `ReceiptRejected` Typed Refusal

Under **Rule 18** of `AGENTS.md`, `ReceiptRejected` is a bounded typed refusal code triggered when a structural receipt fails internal validation. Typical failures include cryptographic digest mismatches, structural bound violations, or temporal window constraints (such as unsatisfied mode dwell times).

In the deterministic `bcinr` substrate, execution in the hot path is strictly governed by the **Radon Law ($CC=1$)**, meaning absolutely no data-dependent loops, conditional jumps, or early returns are permitted. To evaluate a receipt and trigger `ReceiptRejected`, the substrate employs **branchless arithmetic and bitwise masking**:

1. **Branchless Evaluation**: Instead of an `if actual == expected` check, equality is evaluated using bitwise XOR. For temporal inequalities (e.g., $elapsed \ge required$), the system extracts the sign bit using two's-complement wrapping subtraction (`const_lt_u32`), outputting a strict `1` (invalid) or `0` (valid) without branches.
2. **Canonical Refusal Masking**: This single bit is transformed into a full-width bitmask via wrapping arithmetic (e.g., `0u32.wrapping_sub(condition & 1)`). An error bit of `1` yields an all-`1`s mask (`0xFFFFFFFF`), while `0` yields `0x00000000`.
3. **Aggregation (Union)**: Using this mask, the system mathematically embeds the `ReceiptRejected` flag into a global `RefusalSet` using a bitwise `OR` (`union`). If the receipt was valid, the all-`0`s mask ensures nothing is added.

### Relationship to the `ReceiptSound` Law (Rule 11)

**Rule 11 (`ReceiptSound` Law)** dictates that any adaptive state mutation *requires* a strict set of accepted receipts (e.g., `AdmittedControlState`, `AcceptedCertificate`, `AcceptedEnvelopeReceipt`, `AcceptedOutcomeReceipt`, `CertifiedLearningMode`). It explicitly mandates:
> *"No alternate constructor or API may exist... The frozen fallback must be implemented by masked state selection, not branching."*

The `ReceiptRejected` refusal acts as the deterministic enforcer of this law:

- **Mathematical Enforcement**: When an operation attempts adaptive mutation, the validity of its incoming receipts is verified using the branchless evaluation above. If any receipt fails, `ReceiptRejected` is written into the `RefusalSet`.
- **Mask-Based State Transition**: Because Rule 11 forbids `if invalid { return Err(...) }` before committing state, mutations rely on bitwise multiplexers (like `const_select_u32`). The presence of `ReceiptRejected` in the `RefusalSet` dynamically directs this multiplexer to reject the candidate mutation and clone the exact previous state bit-for-bit.
- **Strict Compliance**: This pipeline guarantees that uncertified updates mathematically cannot mutate persistent state, while ensuring the execution path strictly satisfies the O(1) time complexity and allocation-free ($CC=1$) bounds required by `ReceiptSound`.
