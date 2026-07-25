# POWL Typestate Pattern in the Slow Rail Compilation Pipeline

The `bcinr-powl` crate employs a robust Typestate design pattern in its Slow Rail compilation pipeline to guarantee type-safe, deterministic execution of Partially Ordered Workflow Language (POWL) graphs. By mapping state transitions to Rust's type system, invalid state transitions and out-of-order execution become impossible at compile time.

## 1. Logical Evolution of Structures

The pipeline logically evolves hierarchical, untrusted structures into bounded, branchless, allocation-free execution environments.

### Step A: Unverified AST Nodes (`PowlAstNode`)
The workflow begins as a hierarchical Abstract Syntax Tree (`PowlAstNode`). This structure represents sequences, partial orders, XOR choices, and loops. At this stage, the graph is untrusted and may contain structural flaws such as disconnected subgraphs, cycles, or unbound loops.

### Step B: Flat Allocation to `PowlTape`
The Slow Rail recursive descent compiler traverses the AST and flattens it into a `PowlTape`. 
- **Array-backed Structure**: The tape uses fixed-size arrays (up to 64 `Powl64Op` slots).
- **Bitmask Wiring**: Edges are encoded as branchless `pred_mask` and `succ_mask` bitmasks instead of pointer-based references.

### Step C: Structural Graph Verification
Before execution, the `PowlTape` must pass stringent algorithmic verification in the Slow Rail:
- **Cycle Detection**: Kahn's algorithm performs a topological sort to guarantee that the graph is free of non-loop cycles.
- **Reachability Validation**: A Bit-Parallel Transitive Closure Reachability Validation (BP-TCRV) uses the Roy-Warshall algorithm via constant-time bitwise operations to ensure no unreachable paths exist.

## 2. The `PowlRunner` Typestate Machine

Once a structurally sound `PowlTape` is generated, it enters the `PowlRunner` typestate machine. The `PowlRunner` prevents out-of-order operations by consuming `self` on every state transition. 

The pipeline defines five zero-sized compile-time phase markers:

1. **`Unvalidated`**: Initial phase (`PowlRunner<Unvalidated>`). The tape's structural properties (empty checks, size bounds, and entry op presence) have not been formally cleared for the runner.
2. **`Compiled`**: Advanced via `.validate()`. Confirms the tape size ($\le$ 64 ops), ensures an entry point exists, and validates there are no structural faults preventing execution.
3. **`Scheduled<KIND>`**: Advanced via `.schedule::<KIND>()`. Associates the runner with a specific execution topology (e.g., `TopologyKind::Standard`, `TopologyKind::Priority`).
4. **`Executing<KIND>`**: Advanced via `.begin_execution()`. The runner begins executing and issues a linear `ExecutionToken`.
5. **`Receipted<KIND>`**: Advanced via `.complete(token)`. A terminal phase generating an immutable `Receipt` providing a cryptographically sound audit trail of the run.

## 3. Zero-Cost Compile-Time Traits and Const Generics

The system relies on zero-cost abstractions to enforce rules without runtime penalty:

- **The `HasPowlTape` Trait Bound**: 
  A capability trait ensuring the tape can securely report its `op_count()`, its `entry_mask()`, and its 32-byte `content_hash()`. It guarantees uniformity when switching between standard tapes and compact v2 tapes without dynamic dispatch.
  
- **`const KIND: TopologyKind`**:
  Topology (priority, standard, background) is threaded through the later phases using const generics (e.g. `PowlRunner<Scheduled<{ TopologyKind::Standard }>, Tape>`). This structurally prevents mixing priorities or running operations under the wrong constraints.

## 4. Branchless Linear Execution Tokens (BLET)

To emulate linear type behavior during the `Executing` phase, the runner yields an `ExecutionToken`. 

- **Linear Resource Emulation**: It does not implement `Clone` or `Copy`. It must be explicitly consumed via `runner.complete(token)`.
- **Branchless Defect Accumulation**: As execution occurs (`token.consume_op()`), any defects like double-firing, out-of-bounds firing, or malformed masks are accumulated branchlessly into bit-flags (e.g. `defect_double_fire |= target_valid ^ present;`).
- **Destructor Bomb**: If dropped while ops remain unfired, a destructor `Drop` implementation forces a panic (in debug builds), ensuring exactly-once task completion across the tape graph.
