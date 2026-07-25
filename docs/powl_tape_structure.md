# PowlTape Structure and Compilation Format

The `PowlTape` is the core structural representation of a compiled workflow in `bcinr-powl`. It provides a flat, zero-allocation, cache-line-aligned array of operations that are executed by a branchless SWAR (SIMD-within-a-register) scheduler. True to the `bcinr` mandate ($CC=1$), this structure is designed for constant-time bitwise operations, mathematical boundedness, and immunity to timing side-channels.

## Binary Representation (v2)

The execution tape architecture is organized around the 64-byte `Powl64Op` structure and bitmask-based graph representations. 

### 1. The Cache-Line-Aligned `Powl64Op`

Each node in the graph is represented by a `Powl64Op` struct, specifically sized and aligned to consume exactly one 64-byte CPU cache line (`#[repr(C, align(64))]`).

```rust
pub struct Powl64Op {
    pub pred_mask: u64,     // Predecessor ops that must complete before this op
    pub succ_mask: u64,     // Successor ops this op activates on completion
    pub ctrl: u64,          // Control word (u64::MAX signals concurrency marker)
    pub op_kind: OpKind,    // Semantic kind (Activity, Silent, XorChoice, Loop, etc.)
    pub choice_group: u8,   // XOR choice-group identifier
    pub depth: u8,          // Nesting depth in the POWL hierarchy
    pub fan_out: u8,        // Outgoing edge count
    pub _pad: [u8; 36],     // Explicit padding to reach 64 bytes
}
```

### 2. Tape Envelopes

The compiled operations are stored in fixed-size arrays. Depending on the size of the plan, two tape forms are supported:
- **`PowlTape` (Standard)**: Holds up to 64 operations (`ops: [Powl64Op; 64]`). `pred_mask` and `succ_mask` fit exactly into single `u64` bitmasks, enabling $O(1)$ branchless scheduling. It tracks `len`, `entry_op`, and `exit_op`.
- **`PowlTapeLarge` (Extended)**: Holds up to 512 operations. Predecessor and successor tracking expands into bitmask arrays (`[[u64; 8]; 512]`).

### 3. Auxiliary Data Structures

- **`LabelSlab`**: A bounded 1024-byte arena storing interned UTF-8 string labels `[u16-len-le][utf8-bytes]...`.
- **`ConcurrencyGuardTable`**: A side-table storing the minimal nonfaces (subsets of tape slots forbidden from firing simultaneously).

---

## Compilation Format: PDDL Plan to Branchless POWL Tape

The `bcinr-powl` compilation engine translates a source model into the flat `PowlTape`. There are two compilation paths: the legacy Abstract Syntax Tree (AST) recursion and the V2 Plan compilation (which transforms planner-output `PowlModel` directly).

### V2 Plan Compilation (PDDL `PowlModel` → `PowlTape` v2)

The V2 compiler natively maps planner output (a partial-order plan) to the branchless tape execution layer. The structure is inherently flat and relies on topological guarantees:

1. **Dense Identification Mapping**: The source `PowlModel` has one node per `ActionOccurrence`. Nodes must be densely packed and 0-indexed (`node.id() == position`). The numerical ID maps *directly* to the `PowlTape` slot index, allowing O(1) array access without translation tables.
2. **Precedence Wiring**: The PDDL plan's DAG edges are encoded into the tape using bitwise ORs. If action `A` (index 0) precedes action `B` (index 1), the compiler sets the 0th bit in `B`'s `pred_mask` and the 1st bit in `A`'s `succ_mask`.
3. **Concurrency Re-Keying**: The planner outputs a conflict witness matrix (nonfaces that violate mutual exclusion). The compiler re-keys these `EventSet`s from the causal plan's occurrence space directly into the tape-slot bitmask space. These are stored in the `ConcurrencyGuardTable` to branchlessly gate concurrent evaluation ticks.
4. **Label Interning**: PDDL action names are interned into the `LabelSlab`, mapping `PowlNodeId -> u16` offset, which keeps strings out of the 64-byte hot-path `Powl64Op`.

### The Two-Phase Verification Protocol

Every successfully compiled tape is run through rigorous deterministic validation gates before it can transition to execution:

1. **Cycle Detection (Kahn's Algorithm)**: Computes in-degrees (excluding loop back-edges) and performs a breadth-first traversal. If fewer nodes are visited than exist on the tape, a non-loop cycle is detected, failing compilation.
2. **BP-TCRV (Bit-Parallel Transitive Closure Reachability Validation)**: Computes the transitive closure branchlessly using a 64x64 fixed-bound bitwise Roy-Warshall algorithm (`O(V^3 / 64)`). It guarantees via Hoare logic that every active execution node is structurally reachable from the `entry_mask`.

By converting sequential semantic decisions into masks and arithmetic logic, the output `PowlTape` is structurally guaranteed to evaluate sequentially in strict timing-invariant cycles.
