# MAPE-K Autonomic Loop in the Deterministic Substrate

The MAPE-K (Monitor-Analyze-Plan-Execute over a shared Knowledge base) Autonomic Loop is the core self-managing mechanism in the BCINR architecture. Governed by the stringent project mandates in `GEMINI.md`, the loop strictly adheres to the absolute runtime laws: zero heap allocations (`#![no_std]`) and the Radon Law ($CC=1$). This ensures the autonomic execution logic functions as an axiomatic calculus expressed purely through arithmetic, completely immune to timing side-channels.

## Implementation via `AutonomicSubstrate`

All self-managing components must utilize the `AutonomicSubstrate` building blocks to implement the 5-step loop, meticulously aligning with the substrate's principles of branchless execution and deterministic behavior.

### 1. Observe
- **Mandate:** Collect bit-level telemetry.
- **Implementation:** During this phase, the system ingests raw bit-level telemetry data without any dynamic memory allocation, parsing, or unbounded iteration that would introduce branches. Telemetry is safely and directly updated into fixed-size structures (the shared Knowledge base) within the `AutonomicSubstrate` in constant time.

### 2. Infer
- **Mandate:** Calculate `RlState` using branchless metrics.
- **Implementation:** The loop transforms raw observed telemetry into a high-level internal Reinforcement Learning state (`RlState`). To comply with the $CC=1$ requirement, metrics must be derived exclusively using straight-line arithmetic, SWAR (SIMD Within A Register) mechanics, and bitwise polynomials rather than conditional branching.

### 3. Propose
- **Mandate:** Generate `AutonomicAction` masks.
- **Implementation:** Instead of dynamically enqueuing tasks or executing variable control flow decisions, the subsystem computes constant-time, fixed-width execution masks (`AutonomicAction` masks). These bitwise masks encode the intended self-correction or operational adjustments (e.g., Repair, Optimize, Scale), preparing the system for unconditional execution.

### 4. Accept
- **Mandate:** Filter through the `PolicyGuard`.
- **Implementation:** Proposed action masks are evaluated against hard, deterministic boundaries. The `PolicyGuard` enforces strict invariant limits by calculating acceptance through mask logic. If an action is rejected, the resulting mask mathematically zeros out the proposed changes (producing an all-zeros mask). An accepted action produces a full-width positive mask, strictly avoiding speculative branching or early returns.

### 5. Execute
- **Mandate:** Advance state via constant-time transitions.
- **Implementation:** The system's persistent state inside the `AutonomicSubstrate` is advanced using the accepted masks. The state transition relies exclusively on mask-based selection, e.g., `next_state = select(mask, proposed_state, current_state)` or bitwise equivalents like `(mask & proposed) | (~mask & current)`. This guarantees execution in strictly deterministic time without data-dependent instruction paths, back-edges, or runtime theorem discovery.
