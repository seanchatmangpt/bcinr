# bcinr-powl: Partially Ordered Workflow Language Engine

The `bcinr-powl` crate is a proof-carrying, deterministic execution engine for Partially Ordered Workflow Language (POWL) workflows. True to the `bcinr` project mandate (the "deterministic substrate"), the POWL runtime features zero heap allocations on the hot path, mathematically bounded latency, and branchless evaluation ($CC=1$). 

It co-produces cryptographic audit receipts alongside every execution, enabling mathematically provable conformance checking against a reference tape, and natively exports to OCEL 2.0 for process mining.

## 1. The POWL AST and Compilation

Workflows in `bcinr-powl` are modeled via the `PowlAstNode` enum, combining block-structured constructs and explicitly defined partial order dependencies:

- **`Atom(&'a str)`**: A named concrete activity transition.
- **`Silent`**: A silent/tau transition (executes with no side effects).
- **`Sequence(Vec<PowlAstNode>)`**: Strict left-to-right sequential composition.
- **`PartialOrder { children, edges }`**: Directed acyclic dependencies (DAG). 
- **`XorChoice(Vec<PowlAstNode>)`**: Exclusive branch selection (XOR split-join).
- **`Loop { body, redo, max_iters }`**: Bounded iteration (evaluated using saturating arithmetic to avoid branching).

The compiler (`compiler.rs`) recursively transforms this AST into a flat execution tape (`PowlTape`). XOR choices inside loops are structurally forbidden by the compiler (`CompileError::XorInsideLoop`), guaranteeing bounded determinism and guaranteeing a stable audit receipt.

## 2. Execution Tape and OpKinds

The compiled `PowlTape` contains up to 64 operations in a flat array, strictly optimized for cache-line density and SIMD/SWAR processing.

Each slot in the tape is a `Powl64Op` storing:
- `pred_mask`: Bitmask of predecessor slots that must be `done` to enable this operation.
- `succ_mask`: Bitmask of successor slots that will enter the `check_mask` when this operation completes.
- `kind`: The transition discriminant (`OpKind`).

**Supported OpKinds**:
- `Atom` (0): Concrete task.
- `Silent` (1): Evaluated implicitly.
- `XorDispatch` (2): Selects exactly one live branch.
- `Join` (3): Waits for predecessor convergence.
- `LoopRedo` (4): Loop back-edge re-enabling a loop body.

## 3. Phase-Indexed Typestate Machine

Workflow lifecycle transitions are enforced at compile time using a Phase-Indexed Typestate Machine (`typestate.rs`). State is encoded via the `PowlRunner<Phase>` struct, where phase transitions consume `self` by value, making out-of-order execution impossible:

1. `Unvalidated`: Raw unverified tape structure.
2. `Compiled`: Tape passed structural checks (cycles, bounds, connectivity).
3. `Scheduled<KIND>`: A runtime topology is assigned (tracked as a const generic `TopologyKind`).
4. `Executing<KIND>`: The active workflow yielding linear execution tokens.
5. `Receipted<KIND>`: Terminal phase where a cryptographically sound `Receipt` is issued.

The `TopologyKind` governs the priority logic: `Priority`, `Standard`, `Background`, `LongRunning`, or `Compensating`.

## 4. Branchless SWAR Scheduler

At the heart of the engine is the branchless SWAR (SIMD-within-a-register) scheduler (`scheduler.rs`). It operates entirely via bitwise operations and bitmasks to execute control flow without triggering branch prediction or timing side channels.

The execution state is governed by a `PowlRunState`:
- `check_mask`: Operations ready to be evaluated.
- `done_mask`: Operations successfully completed.

**The Scheduler Tick Pipeline:**
1. **Readiness Evaluation**: Evaluate candidate slots using bitwise `trailing_zeros` (constant time). A candidate is ready if `(pred_mask & !done_mask) == 0`.
2. **Firing Selection**: Candidate slots that meet the predicates generate a `fire_mask`.
3. **State Commit**: Fired slots bitwise-OR into the `done_mask`. Successors defined by their `succ_mask` are OR'd into the `check_mask`.
4. **Control Dispatch**: `XorDispatch` and `LoopRedo` rules branchlessly clear obsolete subsets and restart loop bodies based on bitwise polynomials.

Because the maximum tape size is bounded at 64 ops, the scheduler achieves an $O(C)$ algorithmic time complexity (where $C \le 64$ operations per tick).

## 5. Concurrency-Aware Guarding

The scheduler supports concurrency constraints via a `scheduler_tick_guarded` mechanism. It utilizes a `ConcurrencyGuardTable` and a `ConcurrencySelector` to compute the maximal compatible subset of operations that can fire in parallel, branchlessly preventing mutual exclusion violations in complex topologies.

## 6. Branchless Linear Execution Tokens (BLET)

Once a runner transitions into `Executing`, it yields an `ExecutionToken`. Emulating linear type semantics in Rust, this token must be systematically fed execution tasks and completely consumed to close the execution frame.

Defects, such as double-firing or missing an activity out of bounds, do not panic or branch instantly. Instead, defect states accumulate via bitmasks in the token's internal registers. Once execution terminates, the transition to `Receipted` enforces validation branchlessly. Defective runs return an `ExecutionDefect` and safely discard the commit without corrupting system state.

## 7. Audit Receipts and OCEL Conformance

When execution successfully transitions to the `Receipted` state, the runtime issues a `Receipt`. The receipt encapsulates:
- `op_trace`: Bitmask of all operations that executed.
- `topo_order`: Firing sequence graph.
- `chain_hash`: BLAKE3 hash proving execution integrity.
- `overflow`: Records dropped events if the event ring buffer saturated.

Off the hot path (when using `--features std`), the engine natively exports these receipts to Object-Centric Event Logs (OCEL 2.0). The `validate_against_tape` function consumes this data to deterministically assert conformance checking between generated event logs and the original bounded mathematical workflow model.
