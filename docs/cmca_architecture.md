I have inspected `crates/bcinr-cmca/src/lib.rs`. Here is the markdown documentation summarizing the overall architecture and data flow based on its root module:

# CMCA Architecture & Data Flow

Based on the documentation in the root module (`crates/bcinr-cmca/src/lib.rs`), the `bcinr-cmca` (Covariance Monitoring and Calibration Assessment) crate is an authoritative, deterministic systems library designed for runtime telemetry, calibration, and monitoring of bounded computational systems.

## Core Architecture

The crate is built strictly around the **Radon Law** (Cyclomatic Complexity $CC=1$). Its architecture guarantees deterministic execution by auditing the health and stability of mathematical substrates without introducing:
- Timing side-channels
- Dynamic heap allocations
- Panic-prone execution paths

## Data Flow (MAPE-K Loop)

The data flow is structured as an autonomic feedback loop based on the standard **MAPE-K** (Monitor, Analyze, Plan, Execute, Shared Knowledge) model:

1. **Observe**: The system collects telemetry metrics such as scaling factors (`s_meas`, `s_leaf`), condition bounds (`kappa_hat`, `kappa_under`), eigenvalue bounds (`gamma_min_plus`), and divergence/drift metrics (`d_js`). These are typically structured into a `MeasurementArtifact`.
2. **Infer**: The `observatory` engine computes safety flags branchlessly using fixed-point arithmetic invariants.
3. **Propose**: The system outputs validation states (`ObservatoryFlag`), such as `RecertificationCandidate`, which indicate whether the system is safe to proceed or needs to trigger recertification.

## Key Modules

The architectural pipeline relies on these primary modules:
* `fixed`: Provides Q16.16 fixed-point arithmetic primitives tailored for $CC=1$ operations.
* `allocator`: Implements linear bounds-checked and panic-free bump allocators to stay entirely allocation-free in the hot path.
* `observatory`: The evaluation engine that computes calibration safety flags based on the collected mathematical thresholds.
