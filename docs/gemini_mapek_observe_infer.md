# MAPE-K Autonomic Loop: Observe and Infer Phases

The MAPE-K (Monitor-Analyze-Plan-Execute over a shared Knowledge base) Autonomic Loop is a fundamental self-managing mechanism in the BCINR "Deterministic Substrate." It strictly adheres to architectural mandates like the **Radon Law ($CC=1$)** and **Zero-Allocation Boundary** (zero heap allocations, `#![no_std]`).

All self-managing components utilize the `AutonomicSubstrate` to implement this loop via constant-time, branchless operations.

## 1. Observe Phase: Collecting Bit-Level Telemetry
During the **Observe** phase, the system collects and ingests bit-level telemetry.
- **No Branching or Allocation**: The ingestion of telemetry strictly avoids dynamic memory allocation and parsing logic that would introduce conditional branches or timing side-channels.
- **Constant-Time Storage**: The raw bit-level data is updated directly into fixed-width, pre-allocated memory structures—specifically the "Knowledge" repository (implemented as a `PackedKeyTable` inside the `AutonomicSubstrate`). This ingestion is completed in constant time ($O(1)$) regardless of the telemetry values.

## 2. Infer Phase: Calculating `RlState` with Branchless Metrics
In the **Infer** phase, the system analyzes the observed telemetry to calculate the internal Reinforcement Learning state (`RlState`).
- **The `RlState` Structure**: `RlState` is a stack-allocated, zero-allocation container that occupies exactly 136 bits. It is modeled structurally as two 64-bit integers (`low`, `high`) and an 8-bit integer (`extra`) to eliminate heap churn.
- **Branchless Metrics**: Instead of using control flow (like `if`/`else` or data-dependent loops) to categorize telemetry, the system calculates `RlState` using straight-line branchless metrics. This involves bitwise polynomials, SWAR (SIMD Within A Register) mechanics, and arithmetic masks.
- **Constant-Time State Merging**: Transitions and internal calculations of the `RlState` utilize deterministic bitwise operations (e.g., XORing states or fieldwise masked selection). This guarantees the structural integrity of the inference logic while maintaining cyclical complexity at strictly $CC=1$.
