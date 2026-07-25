# Certificate Validation and Mathematical Bounding

In the BCINR substrate, the concept of `AcceptedCertificate` is realized through `CertificateReceipt` (sealed via `seal_certificate` in `crates/bcinr-cmca/src/certification.rs`). The certification process strictly adheres to the autonomic branchless rules.

## 1. Structural Validation

Certificates are structurally validated by checking exactly 11 domain-specific bindings without partial or "mostly matches" logic. The bindings verified include:
- `admitted_graph`
- `generated_payload`
- `kernel_specialization_identity`
- `numeric_profile`
- `q_registry`
- `pricing_law`
- `floor_law`
- `control_mode`
- `influence_state`
- `comparison_derivation`
- `round_identity`

The `seal_certificate` function enforces this by ensuring that the `actual` computed bindings exactly match the `expected` bindings. Any mismatch triggers an immediate, typed deterministic refusal (e.g., `CertificationRefusal::AdmittedGraphMismatch`), completely bypassing any adaptive or branching control flow. 

## 2. Mathematical Bounding (Without Branches)

Before a certificate is sealed, the candidate's state transitions are mathematically bounded by independently recomputing the stability witness:
$$ G \cdot d \le (1 - \delta) d $$

This logic is implemented in the `witness_holds` function:
- **Fixed-Point Arithmetic:** It scales the values by `crate::stability::SCALE` and uses `i128` arithmetic to completely avoid floating-point non-determinism and runtime overflows. 
- **Branchless Bounding:** The matrix operations over $G$ (the graph/matrix) and $d$ (the dimensions) iterate over a fixed dimension (`crate::stability::DIM`). According to BCINR's branchless laws (Rule 13), fixed-bound loops over `DIM` are macro-unrolled or strictly unrolled by the compiler.
- **Constant-Time Verification:** The inequality $gd \le bound$ is evaluated for every row. Sequential decisions become arithmetic proofs rather than complex dynamic discovery paths, ensuring `CC=1` and a fixed object-code trace in the final authoritative assembly.
