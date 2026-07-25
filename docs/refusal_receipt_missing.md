# `ReceiptMissing` Refusal

## Definition
In the `bcinr` deterministic substrate, the `ReceiptMissing` typed refusal is triggered when an authoritative operation attempts to perform an adaptive state mutation without providing all the cryptographically bound proofs (receipts) mandated by the mathematical contract.

## Branchless Mathematical Condition
According to the **`ReceiptSound` Law (Rule 11)**, an adaptive mutation is mathematically uninhabited unless the caller simultaneously supplies specific proof-carrying types:
- `CertificateReceipt`
- `EnvelopeReceipt`
- `OutcomeReceipt`
- `AdmittedControlState` (in `certifiedLearning` mode)

Under the strict anti-branching constraints (Radon Law, $CC=1$), the absence of these receipts cannot be checked using traditional control flow (e.g., `if missing { return Err(ReceiptMissing); }`). Instead, it is evaluated via a branchless transaction pipeline:

1. **Mask Generation**: The C3 Chain evaluates the presence of the required opaque tokens. If any token is missing or if internal digest bindings mismatch, the bitwise **admission mask** ($m_{\mathrm{admitted}}$) completely collapses to `0`.
2. **Deterministic Commit**: The state mutation is executed via a fixed-width `select` operation:
   $$x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t)$$
   If $m_{\mathrm{admitted}} = 0$, $x_t$ is assigned over $x_t$, ensuring the state remains bit-for-bit unchanged.
3. **Yielding the Refusal**: The deterministic function subsequently maps the collapsed mask state ($m_{\mathrm{admitted}} = 0$) to yield the bounded typed refusal code `ReceiptMissing` without early returns or branches.
