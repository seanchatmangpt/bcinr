# bcinr-powl API reference

`bcinr-powl` is the process-model crate: POWL 2 process models, their
compilation to fixed-capacity "tape" bytecode, branchless/lock-free
schedulers that execute the tape tick-by-tick, workflow (WF-net) conversion
with typed refusals, and an OCEL event log with conformance checking.

## Model construction (`process_toolkit`, `powl2`)

- `process_toolkit` builds `Powl2Model` trees: `activity(label)`,
  `silent()`, `sequence(children)`, `partial_order(children, edges)`, plus
  choice graphs and do/redo loops; node identity is a `ProcessNodeRef`
  path (`root()`, `child(index)`, `stable_id()`). Metrics via
  `ProcessMetrics`, structural diffs via `ProcessDiff`.
- `powl2::validate_powl2(model)` checks arity minima
  (`generated_arity` constants), edge validity, acyclicity, and
  choice start/end path constraints; failures are typed `Powl2Error`s.
- `process_rewrite`: semantics-preserving rewrites with digest-pinned
  witnesses — `prepare_process_patch` / `apply_process_patch`,
  `replace_process_node`, `map_activity_labels`,
  `reduce_transitive_process_edges`, `eliminate_redundant_silent_nodes`;
  stale-target CAS refusals surface as `ProcessRewriteError::StaleTarget`.

## Compilation to tape (`compiler`, `tape`, `const_scheduler`)

- `PowlTape` (`tape.rs`) is a fixed 64-op array of `Powl64Op`s with
  predecessor/successor masks, an entry mask, deadlines, per-op max
  durations, and a `LabelSlab` string interner (`intern`, `get`).
- `compile_powl(root)` compiles a POWL AST (`PowlAstNode`) to a tape;
  `compile_powl_v2(model)` compiles a `PowlModel` to `CompiledPowlV2` with a
  `ConcurrencyGuardTable`. Structural failures are typed
  (`CompileError::Cycle`, `XorInsideLoop`, `Unreachable`, ...).
- `check_full_graph_acyclic`, `check_all_ops_reachable`,
  `bp_tcrv_validate_reachability` are the post-compile validation passes.
- `const_scheduler`: compile-time topological orders (`topo_order`,
  `linear_chain_preds`, `parallel_spo_preds`) and `const_tick` /
  `static_tick` execution for statically schedulable topologies.

## Scheduling and execution (`scheduler`, `scheduler_v2`, `scheduler_wide`, `scheduler_wired`)

- `scheduler_tick(tape, state)` advances a `PowlRunState` one tick,
  returning a `FiredSet`; resource booking via `ResourceRegistry`
  (`book_interval`, `check_conflict`) over `OpTimeInterval`s; loop bounds
  tracked per-op (`loop_iters`, `iter_under_limit`).
- `scheduler_v2`: `scheduler_tick_v2` with a pluggable
  `ConcurrencySelector` and guard table; outcomes are typed
  `PowlV2TickOutcome` (`Fired`, `Complete`, `Deadlock { remaining_mask }`).
- `scheduler_wide`: 512-op tapes over `KBitSet<8>` state with a `TimeWheel`
  SLA wheel (`wide_tick`, `WidePowlState`).
- `scheduler_wired`: Petri-flavored tick (`petri_tick`,
  `petri_tick_guarded`) feeding lock-free MPMC rings of `WorkItem`s, plus a
  `FiberPool` for WCET-fiber claims; SLA scheduling via `schedule_sla`.
- `dispatcher`: the lock-free `BpadDispatcher` — `try_submit` / `try_claim`
  return typed `SubmissionResult` / `ClaimResult` with refusal codes.
- `typestate`: the phase-typed state machine over the whole lifecycle
  (`Unvalidated → Compiled → Scheduled → Executing → Receipted`) with
  `ExecutionToken` defect counters and `record_fire`.

## Enterprise operations (`enterprise`, `admit`)

- `admit`: atomic admission control — `AtomicAdmissionParameters` /
  `AdmissionParameters` (`GLOBAL_ADMISSION_PARAMETERS`), tenant class and
  urgency tier extraction (`tenant_class`, `urgency_tier`,
  `has_sla_token`), `DEFAULT_PARAMETERS`.
- `enterprise`: saga-aware op metadata — `EnterpriseOpMeta` (exactly 64
  bytes, one cache line), `SagaStack` (branchless compensating-pop), `SagaRole`,
  `capability_mask(granted, required)`, deadline-carrying
  `deadline_ns` / `sla_tier` fields.

## WF-net conversion and refusals (`wf_net`, `wf_to_powl`, `recompose`, `language`)

- `wf_net::WfNet::new` builds a soundness-checked workflow net
  (`NetError::NotWfNet` / `DanglingArc`); `places`, `transitions`, `source`,
  `sink`, `label` accessors.
- `wf_to_powl::convert_and_verify(net, budget, max_len)` converts a WF-net
  to a `Powl2Model` under a depth budget; every failure is a typed
  `RefusalReason` (`IrreducibleFragment`, `BudgetExhausted`,
  `NotSafe`, `NotSound`, `SoundnessUndecided`, `NotRecomposable`, ...) with
  witness data — the refusal ledger in `docs/contracts/REFUSAL_LEDGER.md`
  documents these classes.
- `recompose::recompose(model)` inverts compilation back to a `WfNet`
  (`RecomposeError::BoundedLoopNotRepresentable` when a bounded loop cannot
  be represented).
- `language`: bounded-language agreement checks — `powl2_language(model,
  max_len)` vs `wf_net_language(net, max_len)`.

## OCEL event log and conformance (`ocel`, `receipt_worker`)

- `OcelLog` records up to 512 `OcelEvent`s per run:
  `record_op_fired`, `record_run_sealed`, resource acquire/release,
  refusals, blocks, and timeouts (typed `EventKind`s).
- `ConformanceResult` enumerates typed violations: `Violation` (missing
  predecessor fired), `ChoiceViolation`, `DuplicateFire`, `SealMismatch`,
  `EventAfterSeal`, `DurationViolation`, `LeaseViolation`,
  `DeadlineViolation`, and more.
- `OcelTraceReceipt` pins a log to a 32-byte `digest()` for replay.
- `receipt_worker`: fixed-capacity chained receipt log (`ReceiptLog`,
  `ReceiptWorker`) with hash chaining (`prev_chain_hash`), bounded pending
  set, and `DrainResult` sealed/refused counts.

## Multifractal consequence mass (`multifractal`)

`cascade_tree(model, mass_of)` builds a `CascadeTree` of per-node mass;
`consequence_mass(model, lenses, mass_of)` evaluates lens-weighted
consequence masses over process nodes (`NonNegativeFixed` values;
`MultifractalError` wraps `ProcessToolkitError` and `CascadeRefusal`).

## Plan projection (`projection`)

`PowlProjector` maps a `CausalPlan` onto a `PowlModel` with verified
preservation: `verify_order_preservation` and
`verify_concurrency_preservation` return witness types, and any dropped or
invented edge/slot is a typed `PreservationError` — projections are refused
rather than lossy.
