# AutonomicAction Masks in the MAPE-K Loop (BCINR)

In the BCINR "Deterministic Substrate," the MAPE-K (Monitor-Analyze-Plan-Execute over a shared Knowledge base) autonomic loop operates under strict architectural constraints. Dictated by the **Radon Law ($CC=1$)** and the **Zero-Allocation Boundary**, the system cannot use variable-length task queues, conditional branches (`if/else`), or dynamic loops.

To trigger system self-management operations like repair, optimization, or scaling, BCINR replaces traditional control-flow and task queuing with **fixed-width bit masks** modeled around `AutonomicAction`. 

## 1. How Masks are Used in Propose/Accept Stages

### Propose (Plan) - Mask Generation
Based on the `AutonomicState` (which encompasses metrics like `drift_detected`, `integrity`, and `health`), the system proposes an `AutonomicAction`. Rather than branching to construct specific task objects or allocating them onto a queue, the `Propose` step mathematically evaluates telemetry to derive fixed-width masks for each potential operation. For example, if integrity drops below a threshold, the mathematical evaluation automatically evaluates to `!0` (all 1s) for the `Repair` action mask.

### Accept - The Policy Guard
Before execution, proposed action masks must pass the `PolicyGuard`. This component strictly evaluates proposals against hard deterministic boundaries. The `PolicyGuard` transforms boolean constraints into masks branchlessly. For example, validating a threshold looks like this:
```rust
let check = (val > threshold) as u64;
let mask = 0u64.wrapping_sub(check); 
// Results in 0xFFFFFFFFFFFFFFFF if true, 0x0000000000000000 if false
```
If an action violates the risk matrix, the guard applies a bitwise `AND` that effectively zeroes out the action mask, rejecting the operation without speculative state mutation or early returns.

## 2. Why Use Full-Width Masks Instead of Enums/Booleans?

BCINR avoids enums (like `ActionKind::Repair` or `Option<Task>`) and booleans for decision-making due to the following core substrate laws:
1. **No Branching ($CC=1$):** Rust enums and booleans typically require `match` statements or `if` blocks to dispatch behavior. This compiles into conditional jumps in the object code. The substrate mandates that the runtime executes the exact same sequence of instructions regardless of whether an action is required or not. 
2. **Zero-Allocation:** Variable task queues require heap allocation. Instead, BCINR uses pre-allocated, fixed-width states and strictly applies bitwise operations.
3. **Execution Uniformity:** By utilizing binary polynomials and SWAR (SIMD Within A Register) mechanics, wide integer masks (`0xFF...FF` for active, `0x00...00` for inactive) allow the CPU to perform operations uniformly. Operations are always "executed" from a CPU perspective, but their results are only conditionally committed to the persistent state.

## 3. Structural Application During the Execute Phase

During the Execute phase, the system uses the accepted masks to advance the state in strictly constant time. Execution occurs through branchless, fixed-width state selection. The transition mathematically blends the `proposed_state` and `current_state` using the action mask:

```rust
// Mask-based state transition without branching
next_state = (mask & proposed_state) | (!mask & current_state);
```

If the action was rejected or unneeded (mask is `0x0`), the `proposed_state` is effectively zeroed out, and the `current_state` is preserved perfectly bit-for-bit. If accepted (mask is `!0`), the system flawlessly transitions to the `proposed_state`. 

By utilizing `AutonomicAction` masks, the `Propose`, `Accept`, and `Execute` steps maintain completely uniform CPU instruction paths. Whether a `Repair`, `Optimize`, or `Scale` action is triggered or ignored, the system executes the exact same fixed amount of work, entirely eliminating timing side-channels, memory allocation variability, and branching complexities.
