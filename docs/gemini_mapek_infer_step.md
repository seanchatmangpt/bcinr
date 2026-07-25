Based on the `GEMINI.md` architectural laws and associated documentation in the `bcinr` codebase, here is the detailed breakdown of the **MAPE-K Autonomic Loop** with a focus on **Step 2 (Infer)** and what it entails for the substrate:

### MAPE-K Autonomic Loop Overview
All self-managing components must utilize the `AutonomicSubstrate` building blocks to implement:
1. **Observe**: Collect bit-level telemetry.
2. **Infer**: Calculate `RlState` using branchless metrics.
3. **Propose**: Generate `AutonomicAction` masks.
4. **Accept**: Filter through the `PolicyGuard`.
5. **Execute**: Advance state via constant-time transitions.

### Step 2: Infer (Detailed Breakdown)
In the **Infer** phase, the system analyzes the raw telemetry collected during the Observe phase to calculate the internal Reinforcement Learning state (`RlState`). To comply with the strict project mandates (the Radon Law where CC=1 and the Zero-Allocation Boundary), this calculation must be performed deterministically and without timing side-channels.

#### What "Calculate `RlState` using branchless metrics" entails:

1. **The `RlState` Structure (Zero-Allocation)**
   - `RlState` is a stack-allocated, zero-allocation container that occupies exactly 136 bits. 
   - It is structurally modeled as two 64-bit integers (`low`, `high`) and one 8-bit integer (`extra`) to completely eliminate heap churn.

2. **Branchless Metrics (Radon Law Compliance)**
   - Instead of relying on control flow constructs like `if`/`else`, `match`, or data-dependent loops to categorize telemetry, the system derives metrics and calculates `RlState` using strictly straight-line arithmetic.
   - This involves using bitwise polynomials, arithmetic masks, and SWAR (SIMD Within A Register) mechanics.

3. **Constant-Time State Merging**
   - Transitions and internal calculations of the `RlState` utilize deterministic bitwise operations (such as XORing states or using fieldwise masked selection). 
   - This guarantees execution in strictly deterministic, constant time ($O(1)$) without any data-dependent instruction paths or back-edges, ensuring the inference logic's structural integrity remains at exactly CC=1.
