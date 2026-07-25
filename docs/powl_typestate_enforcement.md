# POWL Typestate Machine and Topology Enforcements

The `bcinr-powl` crate leverages Rust's type system to enforce strict lifecycle invariants for its Partially Ordered Workflow Language (POWL). At the core of this system is `typestate.rs`, which provides a statically-checked state machine (`PowlRunner`) and zero-cost abstraction for workflow execution.

## Phase-Indexed Typestate Machine

A workflow run goes through exactly five compile-time phases, mapped to zero-sized marker types:

1. **`Unvalidated`**: The initial state of the workflow tape.
2. **`Compiled`**: Reached after the tape passes structural validation (e.g., checking for cycles, entry ops, size limits).
3. **`Scheduled<KIND>`**: The runner is assigned a scheduling topology.
4. **`Executing<KIND>`**: Execution begins, yielding a linear `ExecutionToken`.
5. **`Receipted<KIND>`**: Execution completes successfully, issuing an immutable `Receipt`.

### The Phase Lattice

```text
         [PowlRunner<Unvalidated>]
                     │
                     │ .validate()
                     ▼
          [PowlRunner<Compiled>]
                     │
                     │ .schedule::<KIND>()
                     ▼
       [PowlRunner<Scheduled<KIND>>]
                     │
                     │ .begin_execution()
                     ▼
     ┌──────────────────────────────────────┐
     │  (PowlRunner<Executing<KIND>>,       │
     │   ExecutionToken)                    │
     └──────────────────┬───────────────────┘
                        │
                        │ .complete(token)
                        ▼
      [PowlRunner<Receipted<KIND>>]  +  [Receipt<KIND>]
```

## Compile-Time Lifecycle Enforcements

Rust's type system enforces several execution properties entirely at compile time without runtime branching:

1. **One-Way Transitions (Consuming `self`)**: State transitions are performed by methods that take `self` by value (e.g., `pub fn validate(self)`). This physically consumes the runner in its current phase and returns a new struct representing the next phase, statically preventing accidental reuse of stale states or jumping phases out of order.
2. **Phase Isolation**: Functions are implemented conditionally on the specific phase marker. For example, `.complete()` is only defined on `PowlRunner<Executing<KIND>, Tape>`, enforcing that a workflow cannot complete before it has been scheduled and explicitly started.
3. **Const-Generic Topologies (`TopologyKind`)**: The scheduling behavior (`Priority`, `Standard`, `Background`, `LongRunning`, `Compensating`) is bound to the type itself as a const generic parameter (`const KIND: TopologyKind`) once it enters the `Scheduled` phase. It statically tracks the execution context through the `Executing` and `Receipted` phases. This avoids runtime introspection of workflow types and prevents invalid logic pathways, ensuring context alignment.

## Linear Defect Tracking via Execution Tokens

During the `Executing` phase, execution step correctness is tracked safely in a `no_alloc` environment using an `ExecutionToken`.

- **Linear Type Emulation**: The token purposely omits `Clone` and `Copy`. The compiler ensures the token acts as a linear resource that must eventually be handed back to `.complete()`.
- **Branchless Operational Bounds Checking (`CC = 1`)**: Instead of introducing branches during task execution, defects like double-firing, out-of-bounds fires, or partial fires update stateful bitmasks (`defect_double_fire`, `defect_invalid`, etc.) using branchless bitwise arithmetic. 
- **Defect Isolation**: If a workflow violates invariant states, the defect bitmasks reject the transition to `Receipted` at commit time. This returns an `ExecutionDefect` error while cleanly abandoning the consumed typestate.
