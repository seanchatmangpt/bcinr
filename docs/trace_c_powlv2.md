# DEEP TRACE C — POWL V2: Semantic Edge Archaeology

## 1. Trace Pipeline: PowlProjection -> compile_powl_v2 -> CompiledPowlV2 -> Schedulers

1. **`PowlProjection`**:
   - Resides in `bcinr-mfw-ir/src/projection.rs`.
   - Projects a PDDL CausalPlan + ExecutableConcurrencyComplex into a POWL model.
   - Provides `ActionNodeBijection`, a two-way mapping between `ActionOccurrenceId` (from PDDL causal logic) and `PowlNodeId` (POWL domain).

2. **`compile_powl_v2`**:
   - Resides in `bcinr-powl/src/compiler.rs`.
   - Takes a `PowlModel` (flat graph of `PowlNode`) and outputs `CompiledPowlV2`.
   - **Crucial mapping assumption**: It verifies that `PowlNodeId` values are dense and ascending (`node.id().0 == index as u64`). This allows tape-slot indices in the bitmask representations (e.g. `EventSet`) to directly coincide with `PowlNodeId` values, seamlessly re-keying the `ExecutableConcurrencyComplex` without runtime translation overhead.

3. **`CompiledPowlV2` Output**:
   - Contains `tape: crate::tape::v2::PowlTape`.
   - Contains `guards: ConcurrencyGuardTable`.
   - Contains `node_labels: BTreeMap<PowlNodeId, u16>` pointing to a `LabelSlab`.

4. **Schedulers Entry Points**:
   - `scheduler_tick` and `scheduler_tick_guarded` (`scheduler.rs`): Take `&[crate::tape::Powl64Op]` (legacy 32-byte ops).
   - `petri_tick` and `petri_tick_guarded` (`scheduler_wired.rs`): Take `&[crate::tape::Powl64Op]` (legacy 32-byte ops).
   - `wide_tick` (`scheduler_wide.rs`): Takes `&crate::tape::v2::PowlTapeLarge`.
   - `dispatcher.rs` runtime: Uses a bespoke flat representation with `OpKind { Activity, PartialOrderGate, Concur }`.

---

## 2. Representation Mapping: Legacy `PowlTape` vs `v2/CompiledPowlV2`

### Tape Layout

| Feature | Legacy `PowlTape` | v2 `CompiledPowlV2` |
| :--- | :--- | :--- |
| **Max Capacity** | 64 ops | 64 ops (`v2::PowlTape`), 512 ops (`v2::PowlTapeLarge`) |
| **Op Structure** | `Powl64Op` (32 bytes) | `v2::Powl64Op` (64 bytes cache-line aligned) |
| **Labels** | Encoded implicitly via Atom | Interned via `v2::LabelSlab` and `node_labels` |
| **Entry/Exit** | `entry_mask` (u64) | `entry_op` (u8), `exit_op` (u8) |
| **Concurrency Gates** | Implicit (none or global) | `ConcurrencyGuardTable` natively integrated |

### Operation Layout (`Powl64Op` vs `v2::Powl64Op`)

| Legacy `Powl64Op` (32 Bytes) | v2 `Powl64Op` (64 Bytes) | Semantic Role |
| :--- | :--- | :--- |
| `pred_mask: u64` | `pred_mask: u64` | Slots that must complete before this fires |
| `succ_mask: u64` | `succ_mask: u64` | Slots to update upon completion |
| `branch_mask: u64` | *Removed* | Legacy XOR branching. Handled via gates/ctrl now. |
| `kind: OpKind` | `op_kind: v2::OpKind` | Instruction discriminator |
| `index: u8` | *Removed* | Op index implied by tape array location |
| `branch_count: u8` | *Removed* | Legacy loop upper bounds |
| *Missing* | `ctrl: u64` | Control word (`u64::MAX` = Concur marker) |
| *Missing* | `choice_group: u8` | Group ID for semantic branching |
| *Missing* | `depth: u8` | Nesting depth in POWL hierarchy |
| *Missing* | `fan_out: u8` | Outgoing edges degree |
| Padding (`[u8; 5]`) | Padding (`[u8; 36]`) | Aligning to 32 bytes vs exact 64 byte cache-line |

### OpKinds

- **Legacy `OpKind`**: `Atom`, `Silent`, `XorDispatch`, `Join`, `LoopRedo`
- **v2 `OpKind`**: `Activity`, `Silent`, `XorChoice`, `Parallel`, `Loop`, `StrictPartial`, `ChoiceGraph`, `Concur`

---

## 3. Representation-Independent Scheduler Law

The underlying axiomatic scheduler law is representation-agnostic. The mathematical protocol driving branchless bitwise scheduling must decouple from the memory layout (whether 32-byte or 64-byte blocks).

**The Universal Law of Branchless Evaluation:**
1. **Eligible Set Calculation**: `candidates = check_mask & !done_mask`
2. **Satisfaction Verification**: `sat = pred_satisfied(done, effective_pred)`
   - `effective_pred = (join_effective & is_join) | (pred_mask & !is_join)`
3. **Execution Gate**: `fire_mask = u64::wrapping_sub(0, sat_bit) & op_bit`
4. **State Transition**: `new_done |= fire_mask`
5. **Successor Propagation**: `new_check |= succ_mask` (of fired ops)
6. **Concurrency Filtration**: `stable_selector.select(ready_set, guard_table)` (pure masking step that can wrap any underlying executor).

In the current codebase, this law is heavily coupled to legacy structures. For example, `petri_tick` hardcodes `Powl64Op` (legacy) inside `build_transition_arrays` despite being an algorithmically sound `PriorityPetriEngine` loop. 

---

## 4. EDGE CARDS (Broken Connections)

### [EDGE CARD 1] CompiledPowlV2 Disconnected from Active Scheduling Loops
**Severity**: CRITICAL
**Source**: `compiler::v2::CompiledPowlV2` (`tape: v2::PowlTape`)
**Target**: `scheduler::scheduler_tick` & `scheduler_wired::petri_tick`

**Description**:
The v2 compiler natively outputs a `CompiledPowlV2` struct containing the modern `v2::PowlTape` (64-byte `Powl64Op`). However, the primary hot execution loops (`scheduler_tick`, `scheduler_tick_guarded`, `petri_tick`, `petri_tick_guarded`) explicitly demand `&[crate::tape::Powl64Op]`, which refers to the legacy 32-byte layout. Consequently, any v2-compiled model cannot actually be executed by the highly optimized `PriorityPetriEngine` loop.

**Required Action**: 
The `scheduler` and `scheduler_wired` loops must be upgraded or duplicated to accept `v2::PowlTape` and `v2::Powl64Op`. A unified trait for `OpStream` or distinct implementation blocks for v2 layouts are needed to connect `CompiledPowlV2` to the execution machinery.

---

### [EDGE CARD 2] Dispatcher's `Powl64Op` Re-Definition
**Severity**: HIGH
**Source**: `dispatcher::Powl64Op`
**Target**: `tape::v2::Powl64Op`

**Description**:
In `src/dispatcher.rs`, an entirely separate `Powl64Op` struct and `OpKind` enum are defined inline:
```rust
pub struct Powl64Op { pred_mask: u64, succ_mask: u64, ctrl: u64, op_kind: OpKind }
pub enum OpKind { Activity, PartialOrderGate, Concur }
```
This fails to utilize the canonical `v2::Powl64Op` defined in `tape::v2`. This duplication means the parallel fan-out dispatcher and `SparseEnabledIndex` are disconnected from the actual structures output by `compile_powl_v2`.

**Required Action**:
`dispatcher.rs` must be aligned to consume `crate::tape::v2::Powl64Op` and use `crate::tape::v2::OpKind::Concur` rather than redefining its own subset of the AST representation.
