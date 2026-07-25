The integration logic evaluates the authoritative branchless runtime (`allocate()`) using a Chicago-style TDD approach, directly passing state variables without any mocks. It acts as a bridge testing the interaction of the slow rail (generated constants, proofs) and the hot path. 

Key mechanisms include:
- **Direct Hot-Path Invocation**: Calls the authoritative `allocate()` root directly over fixed, stack-local variables (`weights`, `costs`, etc.).
- **Generated State Injection**: Uses real baseline workloads and constants (`OBJECT_REGISTRY`, `LENS_REGISTRY`, `LAMBDA`, `ETA`) supplied by the slow rail via `bcinr_cmca::generated::case_studies`.
- **Proof Construction**: Feeds real `AdaptiveUpdate<CertifiedLearning>` proofs—satisfying the `ReceiptSound` law—so the operation is admitted by the runtime control plane.
- **Invariant Checking**: Evaluates numeric outputs (e.g. `AllocationOutcome::candidate()`) summing them to verify strict mathematical conservation (e.g. exactly 65536 bits for `NonNegativeFixed::ONE`).
- **Malformation Handling**: Injects out-of-bounds workload factors to observe how the hot path's fixed-point saturating arithmetic handles faults without branching, ensuring global allocations remain conserved and control-plane refusals remain orthogonal to numeric domain faults.
