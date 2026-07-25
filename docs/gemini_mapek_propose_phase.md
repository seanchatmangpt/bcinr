# The "Propose" Phase and AutonomicAction Masks in BCINR

In the BCINR Deterministic Substrate, the MAPE-K (Monitor-Analyze-Plan-Execute over a shared Knowledge base) autonomic loop operates under strict architectural constraints, most notably the **Radon Law ($CC=1$)** and the **Zero-Allocation Boundary**. This means the system is forbidden from using traditional control flow like `if/else` branches, dynamic loops, pattern matching on enums (e.g., `Option<Task>`), or variable-length task queues.

To trigger system self-management operations (like Repair, Optimize, or Scale), the BCINR framework replaces traditional conditional logic and task queuing entirely with **fixed-width bit masks** mapped to an `AutonomicAction`.

## What Are AutonomicAction Masks?

Instead of using booleans or enums to determine whether to execute an action, BCINR generates wide integer masks (such as 64-bit integers). 
- **Active / Triggered:** Evaluates to all 1s (e.g., `0xFFFFFFFFFFFFFFFF` or `!0`)
- **Inactive / Ignored:** Evaluates to all 0s (e.g., `0x0000000000000000`)

By relying on binary polynomials and SWAR (SIMD Within A Register) mechanics, these full-width masks allow the CPU to perform operations uniformly, eliminating any timing side-channels or execution variability. 

## The "Propose" Phase

During the **Propose (Plan)** step of the MAPE-K loop, the system evaluates the inferred `AutonomicState` (which includes branchless metrics like system health, drift, and integrity) to propose an action.

Rather than branching to construct specific task objects or allocating them onto a heap-backed queue, the system *mathematically evaluates* the telemetry to derive fixed-width masks for each potential operation. For instance, if system integrity drops below a threshold, the mathematical expression inherently resolves to `!0` for the `Repair` action mask.

This constraint evaluation is performed completely branchlessly:
```rust
// A boolean threshold check translated directly into a mask without branching
let check = (val > threshold) as u64;
let mask = 0u64.wrapping_sub(check); 
// Yields 0xFFFFFFFFFFFFFFFF if true, 0x0000000000000000 if false
```

## How Masks Replace Conditional Logic

In a traditional architecture, conditional logic dictates whether to commit an action:
```rust
// Prohibited in BCINR (CC > 1)
if valid_action {
    state = proposed_state;
} else {
    state = current_state;
}
```
This produces conditional jump instructions in the final object code.

In BCINR, control flow is replaced by **fixed-width state selection**. From the CPU's perspective, the "work" for the operation is always executed, but the results are only *conditionally committed* to persistent memory. 

During the **Execute** phase, the framework uses the proposed and accepted masks to mathematically blend the states in strictly constant time:
```rust
// Mask-based state transition (CC = 1)
next_state = (mask & proposed_state) | (!mask & current_state);
```

- If the action is unneeded or rejected by the Policy Guard (`mask == 0x0`), the `proposed_state` is effectively zeroed out, and the `current_state` is preserved bit-for-bit.
- If the action is triggered and accepted (`mask == !0`), the system flawlessly drops the `current_state` and adopts the `proposed_state`.

By using `AutonomicAction` masks, the `Propose` and `Execute` steps maintain completely uniform CPU instruction paths. The system executes the exact same fixed volume of work whether a `Repair`, `Optimize`, or `Scale` action is triggered or ignored.
