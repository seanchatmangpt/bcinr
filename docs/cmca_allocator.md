# CMCA Cascade Resource Allocator

## Purpose
The `allocator.rs` module implements the resource allocation engine for the Covariance Monitoring and Calibration Assessment (CMCA) substrate. It distributes resource flows hierarchically down a forest structure of nodes based on semantic mass, policy lenses, and operational prices. 

## Structure & Algorithms
The algorithm processes flow in a fixed number of operations ($N=8$, $K=4$, $Q=4$ models/lenses) across four distinct phases:
1. **Cascade Allocation**: Roots are initialized with weights based on clipped semantic mass. Flow is propagated iteratively down the tree (dividing between direct leaf allocation and child propagation). 
2. **Multiplicative Weights Update (MWU)**: A local divergence metric (relative entropy) between child allocations and subtree distributions dynamically updates the routing weights. This adaptive learning is strongly regulated and requires a valid `AdaptiveUpdate<CertifiedLearning>` proof receipt. 
3. **Stable Projections**: Projected leaf allocations are scaled by operational costs and resource prices ($\exp(-\mu_x \cdot c_x)$) and normalized.
4. **Explore Floors**: A uniform baseline distribution (`eta / n_L`) is mixed in to guarantee minimum search thresholds and prevent numerical singularity.

## Deterministic Constraints (The Radon Law Mandates)
To adhere to the absolute zero-allocation and branchless execution mandate, the allocator strictly enforces:

- **Zero Heap Allocations**: All computations operate strictly on fixed-width stack-allocated arrays (e.g. `[NonNegativeFixed; N]`, `[[NonNegativeFixed; 2 * Q]; N]`). It guarantees $O(1)$ auxiliary stack space.
- **Constant-Time Execution ($CC=1$)**: 
  - There are zero `if`, `match`, or runtime-variable loops. All loops over trees and models are unrolled via static macros like `unroll_8_static!` and `unroll_32_static!`.
  - Conditional logic is replaced with polynomial bitwise operations and bitmask abstractions like `const_select_u32`, `const_lt_u32`, and `select_nnf` which handles conditional selection without hardware jumps.
- **Fault Accumulation and Typed Refusals**:
  - Instead of early returning, unwinding, or panicking upon bounds violations, the allocator evaluates all paths.
  - Anomalies are collected deterministically into a unified bitset structure (`RefusalSet` and `NumericFaultSet`) via branchless bitwise unions.
  - The authoritative root always returns an `AllocationOutcome` object which guarantees the presence of either the exact valid allocation candidate or the accrued set of typed refusal reasons (like `DIGEST_MISMATCH` or `PROPOSAL_REJECTED`), satisfying exact bounded execution execution and maintaining hot-path compliance.
