# AutonomicFeedback in the MAPE-K Loop

In the BCINR substrate, the final phase of the MAPE-K loop incorporates an `adapt` step. This phase is responsible for ingesting scalar rewards (`AutonomicFeedback`) based on the execution phase's outcome and applying them to the system's Reinforcement Learning (RL) state (`AutonomicState`). 

Crucially, in compliance with the Radon Law ($CC=1$) and zero-branch execution mandates, the ingestion and processing of these rewards strictly avoid dynamic branching (`if/else`, `match`).

## 1. Branchless Reward Ingestion (Selection)
Instead of using conditional branches to determine the reward value based on execution success, the `AutonomicKernel::run_cycle` orchestrates a branchless selection utilizing an array-indexing pattern (a fixed-width lookup table). 

```rust
// From `run_cycle` in autonomic/kernel.rs
let reward = [-1.0, 1.0][result.success as usize];
self.adapt(AutonomicFeedback { reward });
```
By casting the boolean `success` flag to a `usize` (evaluating to `0` or `1`), the kernel fetches either a penalty (`-1.0`) or a positive reward (`1.0`) in constant time without control-flow jumps.

## 2. Branchless State Adaptation (Processing)
When the reward is passed to the implementation via the `adapt` method (e.g., in the `Vision2030Engine` reference model), the internal state is mutated structurally through continuous arithmetic rather than conditional logic.

```rust
// From `adapt` in models/vision_2030.rs
fn adapt(&mut self, feedback: AutonomicFeedback) {
    self.state.health = (self.state.health + feedback.reward * 0.01).clamp(0.0, 1.0);
}
```
Here, the `AutonomicFeedback` reward scales a constant learning rate (`0.01`) and alters the `health` state. To ensure the state remains within its valid bounded domain `[0.0, 1.0]` without using bounds-check branches (e.g. `if state.health > 1.0 { ... }`), it employs the `.clamp()` operation. Under the hood, `clamp` compiles down to branchless `min/max` intrinsic instructions, ensuring the execution work remains fixed and bounded.

## Summary
The combination of **array-indexing for state translation** and **intrinsic clamping for boundary enforcement** guarantees that the `AutonomicFeedback` cycle preserves the deterministic, branchless instruction shape required by the BCINR axiomatic calculus.
