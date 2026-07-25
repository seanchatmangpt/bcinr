# MAPE-K Autonomic Loop in BCINR

The MAPE-K (Monitor-Analyze-Plan-Execute over a shared Knowledge base) Autonomic Loop is a core self-managing mechanism in the BCINR "Deterministic Substrate." As mandated by the architecture rules in `GEMINI.md`, the entire loop must adhere to absolute runtime laws: zero allocation, `#![no_std]` compliance, and the Radon Law ($CC=1$), ensuring that execution logic is mathematically branchless and constant-time.

## The 5 Steps of the Autonomic Loop

According to the project mandate, all self-managing components must utilize the `AutonomicSubstrate` building blocks to implement the following pipeline:

### 1. Observe
- **Mandate:** Collect bit-level telemetry.
- **Integration:** Monitors structural data and raw payloads without parsing, allocating, or using unbounded iteration. Telemetry is safely packed directly into fixed-size data structures like the `PackedKeyTable` held within the `AutonomicSubstrate`.

### 2. Infer
- **Mandate:** Calculate `RlState` using branchless metrics.
- **Integration:** Transforms the raw observed telemetry into a high-level Reinforcement Learning state (`RlState`). To comply with the $CC=1$ requirement, metrics must be derived exclusively using straight-line arithmetic, SWAR mechanics, and bitwise polynomials rather than conditional branching.

### 3. Propose
- **Mandate:** Generate `AutonomicAction` masks.
- **Integration:** Rather than enqueuing a variable-length list of tasks or branching on decisions, the subsystem computes constant-time, fixed-width execution masks representing different operations (e.g., Repair, Optimize, Scale). This prepares the system for unconditional execution.

### 4. Accept
- **Mandate:** Filter through the `PolicyGuard`.
- **Integration:** Evaluates proposed action masks against hard, deterministic boundaries. Acceptance is computed mathematically: an accepted action produces a full-width positive mask, while rejection produces an all-zeros mask. This satisfies the rule against speculative state mutation before admission.

### 5. Execute
- **Mandate:** Advance state via constant-time transitions.
- **Integration:** The persistent state inside the `AutonomicSubstrate` is updated by applying the derived masks. Because the state transition relies exclusively on mask-based selection (`next_state = select(mask, proposed_state, current_state)`), it executes in strictly deterministic time without data-dependent instruction paths, back-edges, or runtime theorem discovery.
