# Rule 11 (LearningFrozen Fallback) Research Report

I have completed the research on the `LearningFrozen` fallback mechanism as requested. Here is the markdown documentation based on the architectural documents and source code within the `bcinr` substrate:

## Mechanism Overview

Under **Rule 11 (ReceiptSound Law)**, adaptive mutation requires a strict conjunctive gate (the simultaneous presence of an `AdmittedControlState`, `AcceptedCertificate`, envelope/outcome receipts, and `CertifiedLearningMode`). If any of these required cryptographic receipts are missing, invalid, or fail the $O(1)$ mathematical bounds checks, the requisite proofs for learning are absent. The system structurally falls back to a **`LearningFrozen`** state.

`LearningFrozen` is defined as a bounded typed refusal code (a variant of the `StabilityRefusal` enum in `crates/bcinr-cmca/src/allocator.rs`).

## Branchless Implementation (Masked State Selection)

To comply with the strict zero-branching mandate of the substrate (The Radon Law: $CC=1$), the system cannot use conditional control flow (e.g., `if learning_frozen { return }`). Instead, the freeze mechanism is enforced mathematically via **constant-time, bit-level masked state selection**:

1. **Admission Mask Derivation:**
   The state transition function evaluates the conditions, such as the fixed-point static domination checks against a certified bounding witness. A failed validation structurally zeroes out an admission mask: $m_{\mathrm{admitted}} = 0$.

2. **The Commit Phase:**
   The deterministic commit phase executes an exact bitwise selection multiplexer (e.g., using algorithms functionally equivalent to `(mask & active) | (!mask & fallback)`):
   $$ x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t) $$

   When `LearningFrozen` is triggered ($m_{\mathrm{admitted}} = 0$), the equation evaluates seamlessly to:
   $$ x_{t+1} = \operatorname{select}(0, x_{\mathrm{candidate}}, x_t) = x_t $$

## Impact on State & Selection

By relying entirely on this mask-based commit:
- **Adaptive state fields remain structurally immutable:** The persistent state ($x_t$) is preserved bit-for-bit, enforcing stability without a single `if` statement.
- **Deterministic selection safely continues:** The system continues operating on the existing frozen, bounded parameters ("safe homeostasis").
- **Receipt accumulation:** System receipts continue to accumulate, but without mutating the underlying weights.
- **Recertification is blocked:** No automatic recertification occurs dynamically in the hot path.

*(Primary references: `docs/refusal_learning_frozen.md` and `docs/rule_11_receipt_sound_law.md`)*
