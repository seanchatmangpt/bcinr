# Zero-Cost TypeStates in BCINR

The BCINR codebase utilizes **zero-sized types (ZSTs)** as **TypeStates** to encode workflow and evaluation phases directly into the Rust type system. By statically classifying parameters at compile time, the system guarantees one-way, mutually exclusive state transitions without paying the overhead of runtime checks or data-dependent branching. 

This approach enforces the `CC=1` (Cyclomatic Complexity 1) mandate by transforming what would traditionally be runtime conditional branches (`if state == ...`) into compile-time type boundaries.

## 1. Phase-Indexed Execution (POWL v2)

In the POWL (Partially Ordered Workflow Language) engine (`crates/bcinr-powl/src/typestate.rs`), execution correctness is governed by the **Phase-Indexed Typestate Machine**. The `PowlRunner<Phase, Tape>` struct uses zero-sized markers to track the lifecycle of a workflow tape.

### The Phase Lattice
The runner advances through five distinct compile-time phases, represented by ZSTs:
- `Unvalidated`: Tape structure not yet verified.
- `Compiled`: Tape passed structural validation (no cycles, valid entry ops).
- `Scheduled<KIND>`: Assigned a scheduling topology (e.g., `Standard`, `Priority`).
- `Executing<KIND>`: An execution is in flight, yielding an `ExecutionToken`.
- `Receipted<KIND>`: Execution completed, and an unforgeable OCEL receipt is issued.

### Safety Guarantees
Each state transition consumes the runner by value (`self`) and returns a new runner in the next phase. This produces several zero-cost abstractions:
1. **Phase Isolation**: You cannot call `begin_execution()` on an `Unvalidated` tape, or `schedule()` a runner that is already `Executing`. The compiler strictly enforces method availability.
2. **Consuming Transactions**: Because transitions consume `self`, stale runner states cannot be reused or duplicated, preventing split-brain execution forks.
3. **Erased Overhead**: The ZSTs occupy zero bytes in memory and compile down to direct, branchless code.

## 2. Manufacturing Intelligence (Chess Factory)

In the `chess-factory` cell, TypeStates are used to enforce safe compile-time ordering for evaluation sequences (`SearchState<Phase>`) rather than relying on runtime depth metrics or boolean flags. 

### Phase Over Depth
Traditional chess engines heavily rely on runtime variables like `depth` to dictate search behavior. However, encoding deep metrics into the type system (e.g., `D0`, `D1` ... `D40`) does not scale and requires dynamic branching. 

Instead, BCINR classifies the logic by **Phase**:
```rust
struct SearchState<Phase> { ... }
// Initial → TtProbed → MovesGenerated → MovesOrdered → Resolved
```

### The Chatman Equation in Action
The factory uses these Phase TypeStates to execute the **Chatman Equation** (`A = μ(O*)`). By categorizing game phases (Opening, Tactical, Quiet, Endgame, Tablebase) via zero-sized TypeStates, the correct search graph topology serves as the concurrency specification. 
- Phase TypeStates determine data readiness.
- Work is processed using branchless SWAR (SIMD Within A Register) evaluation.
- No thread spawning or complex `match` branching is needed to route the logic to the correct evaluation kernel.

## 3. Core Advantages of TypeStates in BCINR

- **Branchless Operations**: Eliminates the need for control-flow logic like `match phase { ... }` in the hot path. 
- **0-Byte Abstraction**: The types carry zero memory footprint. `SearchState<Phase>` only stores domain data, completely omitting phase enum discriminants.
- **Proof-Carrying Artifacts**: By pushing the state-machine rules into the compiler, TypeStates function as mathematical proofs. If a workflow compiles, it is structurally impossible for it to violate phase ordering or execute out-of-bounds tasks.
