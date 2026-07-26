# POWL Subsystem Contract

**Version:** 26.7.17  
**Scope:** Process Workflow Language (POWL) IR; compilation from causal plans to executable control flow  
**Responsibility:** Model construction, determinism preservation, tape compilation, execution receipt generation

## Preconditions

### Input: Causal Plan (Pre-Projection)
```
∀ causal ∈ CausalPlan:
  - epoch: PlanningEpochId (unique plan generation marker)
  - occurrences: [ActionOccurrence] where each ActionOccurrence has:
    • id: ActionOccurrenceId (unique within plan)
    • action: u32 (action schema index from domain)
  - precedes: StrictPartialOrder over ActionOccurrenceIds:
    • edges: BTreeSet<PrecedenceEdge {before, after}>
    • ∀ edge, before ≠ after (no self-loops)
    • ∀ paths, acyclic (DAG structure)
  - independence: IndependenceRelation (symmetric, transitive, reflexive)
    • ∀ i, j: independent(i, j) ⟹ i ≰ j ∧ j ≰ i (no ordering edge)
  - digest: BLAKE3 hash of plan structure
```

### Input: Concurrency Complex (Pre-Projection)
```
∀ concurrency ∈ ExecutableConcurrencyComplex:
  - event_count: u32 (cardinality of action set)
  - minimal_nonfaces: [EventSet] (minimal non-faces for concurrency polytope)
  - conflict_witnesses: BTreeMap<Digest, ConcurrencyConflictWitness>
    • Each witness justifies why certain actions cannot run concurrently
    • Witnesses typed: ResourceConflictWitness | TemporalConflictWitness | CausalConflictWitness
  - digest: BLAKE3 hash of concurrency model
```

### Input: Projection Policy (Pre-Compilation)
```
∀ policy ∈ PowlProjectionPolicy:
  - Determines choice points: which branch to schedule first
  - Deterministic: same (causal, concurrency) ⟹ unique POWL model
  - Respects concurrency: projected order ⊆ independence relation
```

### Input: Execution Bounds (Pre-Execution)
```
∀ bounds ∈ ExecutionBounds:
  - max_ops: u8 (≤ 64 operations in tape)
  - max_ticks: u32 (upper bound on execution steps)
  - topology_kind: TopologyKind ∈ [Priority, Standard, Background, Quarantine]
```

## Postconditions

### Successful Projection → PowlModel
```
model ∈ PowlModel ⟹
  ✓ nodes: [PowlNode] with |nodes| = |causal.occurrences|
    • ∀ node, node.id ∈ [0, |nodes|) (dense, 0-indexed)
    • ∀ node ∈ Activity, node.source_action ∈ causal.occurrences
    • ∀ node ∈ Silent, no corresponding action occurrence
  
  ✓ order: StrictPartialOrder extending causal.precedes
    • ∀ edge ∈ causal.precedes.edges, edge ∈ model.order.edges
    • model.order is acyclic (topological sort always exists)
    • order respects independence: if independent(i, j) ⟹ ¬(i → j) ∧ ¬(j → i)
  
  ✓ concurrency: ExecutableConcurrencyComplex passed through unchanged
    • conflict_witnesses preserved exactly
    • event_count matches |causal.occurrences|
  
  ✓ provenance: BTreeMap<PowlNodeId, ActionOccurrenceId>
    • ∀ activity node, provenance maps node to originating action
    • ∀ silent node, no entry in provenance
    • Bijection: ∀ action ∈ causal, exactly one node maps to it
  
  ✓ Projection deterministic: same (causal, concurrency, policy) ⟹ isomorphic model
```

### Successful Compilation → CompiledPowl2
```
compiled ∈ CompiledPowl2 ⟹
  ✓ tape: Powl2Tape with:
    • len: u8, |tape.ops| = len ≤ 64
    • ops: [Powl2Op] where each Powl2Op has:
      - entry_op, succ_mask, pred_mask (bitmasked predecessors/successors)
      - entry: u32 (entry point ID)
      - exit: u32 (exit point ID)
    • ∀ i, succ_mask[i] only set for j where (i → j) ∈ order
    • ∀ i, pred_mask[i] only set for j where (j → i) ∈ order
  
  ✓ Compilation deterministic: same model ⟹ identical tape structure
  
  ✓ Tape is valid:
    • ∀ transition (i → j), both operations defined
    • ∀ op, succ_mask is closed under transitive reduction
    • No cycles reachable: topological ordering exists
```

### Successful Execution → ExecutionReceipt
```
receipt ∈ ExecutionReceipt ⟹
  ✓ executed_ops: [u32] (sequence of operation IDs consumed during execution)
    • ∀ op ∈ executed_ops, 0 ≤ op < len(tape)
    • ∀ consecutive pair (op_i, op_i+1):
      - op_i ∈ succ_mask[op_i-1] (operation reachable from predecessor)
      - ∀ dep ∈ dependencies(op_i+1), dep ∈ executed_ops[..i]
  
  ✓ execution_digest: BLAKE3(tape structure + executed sequence)
    • Deterministic: same execution trace ⟹ identical digest
    • Collision-resistant: different traces have different digests (probability < 2^-128)
  
  ✓ topology: TopologyKind assigned during execution
    • Reflects the execution context (Priority/Standard/Background/Quarantine)
    • Immutable once assigned
  
  ✓ Standing: BLAKE3Verified
    • Receipt cryptographically proven to correspond to tape execution
```

## Invariants

### Maintained Throughout Projection & Execution

1. **Action Bijection**
   ```
   ∀ action ∈ causal.occurrences:
     ∃! node ∈ model.nodes: provenance[node.id] = action.id
   ```

2. **Ordering Preservation**
   ```
   ∀ edge (i, j) ∈ causal.precedes:
     edge ∈ model.order.edges
   ```

3. **Independence Closure**
   ```
   ∀ i, j: independent(i, j) ⟹ ¬∃ path i → j in model.order
   ```

4. **Density (Node IDs)**
   ```
   ∀ node ∈ model.nodes:
     node.id ∈ [0, |nodes|)
     AND ∀ i ∈ [0, |nodes|), ∃ node with node.id = i
   ```

5. **No Cycles**
   ```
   ∀ tape ∈ CompiledPowl2:
     ¬∃ sequence of succ_mask transitions forming cycle
   ```

6. **Execution Trace Validity**
   ```
   ∀ operation sequence in receipt.executed_ops:
     ∀ operation_i, operation_i+1:
       operation_i ∈ succ_mask[operation_i+1] ∨ (all deps satisfied)
   ```

7. **Deterministic Compilation**
   ```
   compile_powl(model) always produces identical CompiledPowl2
     (bit-for-bit, structurally identical)
   ```

## Refusal Conditions

Typed refusal enumeration: `enum ProjectionError { ... }`, `enum CompileErrorV2 { ... }`

### Projection Errors (ProjectionError::*)
```
| PreservationError::UnmappedAction
    Reason: PrecedenceEdge references ActionOccurrenceId not in causal.occurrences
    Recovery: Audit causal plan generation; verify all action IDs present
    
| PreservationError::DependencyViolation
    Reason: Action marked independent but ordering constraint implies ordering
    Recovery: Recalculate independence relation; fix causality analysis
    
| PreservationError::TypeMismatch
    Reason: ConcurrencyConflictWitness references action not in causal plan
    Recovery: Re-analyze concurrency complex; verify witness generation
    
| PreservationError::CyclicDependency
    Reason: Projected order contains cycle (should not occur if causal is acyclic)
    Recovery: Check causal plan for cycles; audit topological sort
```

### Compilation Errors (CompileErrorV2::*)
```
| CompileErrorV2::NodeIdNotDense {position, found}
    Reason: Gap in node ID sequence (e.g., nodes 0, 1, 5 instead of 0, 1, 2)
    Recovery: Verify node.id assignment during projection; ensure [0..n) sequence
    
| CompileErrorV2::TapeFull
    Reason: Model contains > 64 operations; exceeds bitmasked tape capacity
    Recovery: Partition problem into smaller workflows; use hierarchical POWL nesting
    
| CompileErrorV2::InvalidSuccessor {op, succ_id}
    Reason: succ_mask[op] references operation outside [0, tape.len)
    Recovery: Audit successor set construction; verify topological order
    
| CompileErrorV2::CyclicOrdering
    Reason: Tape operations form cycle (should not occur if model.order is acyclic)
    Recovery: Check model.order.edges for cycles; re-run acyclicity check
```

### Execution Errors (ExecutionError::*)
```
| ExecutionError::InvalidOperation {op_id}
    Reason: Attempted to execute op_id ≥ tape.len
    Recovery: Verify operation ID from runtime dispatcher; check tape bounds
    
| ExecutionError::PreconditionNotMet {op_id, missing_dep}
    Reason: Operation missing_dep has not completed before op_id
    Recovery: Enforce predecessor completion; respect pred_mask constraints
    
| ExecutionError::TopologyMismatch {expected, actual}
    Reason: Execution context topology differs from compiled tape topology
    Recovery: Re-compile tape for actual topology; match execution environment
    
| ExecutionError::ReceiptVerificationFailed {digest}
    Reason: Execution trace digest does not match computed BLAKE3 hash
    Recovery: Re-record execution trace; check for trace corruption
```

### Admission Errors (AdmissionError::*)
```
| AdmissionError::BoundsMismatch
    Reason: Projected model.nodes.len() > bounds.max_ops
    Recovery: Increase max_ops bound; or partition workflow
    
| AdmissionError::ResourceExhaustion
    Reason: Topology assignment exhausted for bounds.topology_kind
    Recovery: Use different topology; re-negotiate resource constraints
```

## Semantics

### Projection: Causal Plan → POWL Model

**Goal:** Transform a causally-ordered plan into an explicit concurrency structure (partial order) preserving:
- Action ordering (causal dependencies)
- Independence information (concurrency opportunities)
- Conflict witnesses (why some concurrency is forbidden)

**Process:**
1. Bijection: each ActionOccurrence → PowlNode(Activity)
2. Order: map PrecedenceEdges directly; add transitive reduction if needed
3. Silent nodes: introduced by scheduler for barrier synchronization (minimal)
4. Concurrency: pass through exactly; no modification

**Invariant:** Projection is deterministic (given fixed projection policy)

### Compilation: POWL Model → Executable Tape

**Goal:** Convert the partial-order model into a branchless, bitmasked tape suitable for O(1) execution on heterogeneous topology.

**Process:**
1. Topological sort of model.order
2. Assign dense node IDs in [0, |nodes|)
3. Compute transitive reduction (remove implied edges)
4. Encode predecessors/successors as u64 bitmasks (succ_mask, pred_mask)
5. Wire entry_op (first executable), exit_op (final operation)

**Invariant:** Compilation is deterministic; same model → identical tape

### Execution: Tape → Receipt

**Goal:** Execute operations in a topologically-valid order, recording the sequence as a cryptographic receipt.

**Process:**
1. Initialize execution state (empty executed_ops set, all dependencies tracked)
2. Loop: consume enabled operations (all predecessors executed)
3. Record each operation consumed: executed_ops.push(op_id)
4. On completion: hash (tape, executed_ops) → execution_digest via BLAKE3
5. Assign topology and finalize receipt

**Invariant:** Execution is deterministic; same tape + same operation selection → identical digest

### Topology Kinds

- **Priority:** CPU-bound operations executed on high-priority thread pool
- **Standard:** General workload; fairness enforcement
- **Background:** Low-priority; deferrable operations
- **Quarantine:** Isolated execution; no shared mutable state

Each topology has resource limits and contention profiles; admission gates check feasibility.

## Formal Properties

### Soundness
```
∀ receipt ∈ valid_execution:
  ∀ operation_i ∈ receipt.executed_ops:
    ∀ predecessor_j ∈ dependencies(operation_i):
      predecessor_j ∈ receipt.executed_ops[..i]
```
**Proof:** Execution only allows operation consumption if all predecessors complete; checked at each step.

### Completeness
```
∀ model ∈ PowlModel such that model.order is acyclic:
  ∃ valid_execution such that:
    receipt.executed_ops contains all operations at least once
```
**Proof:** Topological sort guarantees all operations eventually become enabled (no circular dependencies).

### Determinism
```
compile_powl(project(causal, concurrency, policy)) is deterministic:
  Same inputs ⟹ bitwise-identical CompiledPowl2
```
**Proof:** All steps (projection, compilation) are deterministic; no RNG, no choice points.

## Standing

- **Scope:** Process modeling; control-flow compilation; execution receipt generation
- **Cyclomatic Complexity:** CC = 1 (no branching in projection/compilation/execution loops)
- **Allocation:** Stack-only; no heap allocations during tape execution
- **Memory:** O(|nodes|) for bitmasks; ≤ 8 bytes per operation
- **Proof:** Formal verification in `HOARE_TRIPLES.md`; tests in `crates/bcinr-powl/tests/`
