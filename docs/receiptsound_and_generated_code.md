# Interaction between `ReceiptSound` (Rule 11) and Generated Code (Rule 21)

## How Generated Modules Handle Adaptive Mutation Receipts
According to Rule 11 (ReceiptSound law), any adaptive mutation strictly requires all of the following:
1. `AdmittedControlState`
2. `AcceptedCertificate`
3. `AcceptedEnvelopeReceipt`
4. `AcceptedOutcomeReceipt`
5. `CertifiedLearningMode`

Because generated code (Rule 21) operates within the authoritative runtime, generated modules cannot use standard control-flow branching (like `if` statements) to check if these receipts are valid. Instead, they must handle these receipts using **masked state selection** and bitwise polynomial logic to maintain a cyclomatic complexity of exactly `CC=1`. 

If a generated module receives invalid receipts, or if learning is frozen, it must not panic or branch. It must deterministically fall back by continuing selection, leaving adaptive state fields unchanged, and accumulating receipts—all implemented purely via fixed-width masked selection. There can be no alternate APIs or constructors generated to bypass this receipt-validation protocol.

## Why They Must Pass All Authoritative Gates Including Cryptographic Digest Matching
Rule 21 mandates that generated code must be perfectly reproducible, pass all structural gates (like disassembly inspection and cheat scanning), and crucially, **bind to source graph and certificate digests**. This interaction is vital for several reasons:

1. **Cryptographic Anchoring of Logic**: For Rule 11 to hold true, the system must guarantee that the code evaluating the `AcceptedCertificate` and receipts is exactly the code that was mathematically proven. By binding the generated code to the certificate digests, the substrate ensures an unbreakable link between the axiomatic proof and the executed machine code.
2. **Prevention of Unwitnessed Mutation**: Hand-editing generated output is strictly prohibited under Rule 21. If a developer could manually alter the generated module, they could insert logic that bypasses the receipt requirements of Rule 11, mutating state without the required `AcceptedOutcomeReceipt` or `AcceptedEnvelopeReceipt`. The required digest verification strictly prevents this.
3. **Invalidation upon Drift**: If a generated module experiences unexplained drift (its cryptographic digest changes without a corresponding change in the clean generation pipeline), its standing is immediately invalidated. A digest mismatch indicates that the branchless implementation can no longer be mathematically trusted to enforce the five prerequisites of the `ReceiptSound` law, thereby breaking the core deterministic guarantees of the BCINR substrate.

In short, generated modules serve as the crystallized, unalterable enforcers of the `ReceiptSound` law. The cryptographic digest verification is the immutable seal that proves the generated, branchless code will safely evaluate adaptive mutation receipts without any hidden bypasses.
