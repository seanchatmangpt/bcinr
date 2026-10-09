# bcinr-pddl API reference

`bcinr-pddl` carries the planning surface: PDDL 3.1 / PDDL8 parsing, exact
classical grounding and search, temporal execution with trajectory
constraints, a capability router that refuses unsupported feature
combinations, and the consequence/residualizer layer that turns executed
plans into standing-bearing records.

## Admission and capability routing

- `llm_bridge`: admission gate for candidate domain/problem text.
  `admit_candidate_domain(text)` and `admit_candidate_problem(text, domain)`
  return `AdmittedDomain` / `AdmittedProblem` or a `Pddl8Error`;
  `manufacture_world(domain_text, problem_text, case_id, policy_rules)`
  mints a `WorldManufactureReceipt`. `build_ocel_export(plan, case_id)`
  emits an OCEL event JSON for the process-mining side.
- `capability` / `capability_router`: `route_capability_plan(task)` maps a
  `CapabilityTask` to a `CapabilityRouteReceipt` over the declared feature
  profile (`PddlFeature`, `CapabilityProfile`, `unsupported_mask`,
  `AdmittedPlanningTask`); unsupported combinations refuse with a typed
  error instead of degrading silently.
- `problem_builder`: `StripsProblemBuilder` with `PddlObjectBuilder` /
  `PddlAtomBuilder` assembles problem documents programmatically
  (`PddlProblemDocument`), refusing incomplete builds with `PddlBuildError`.

## Exact classical planning (`ground_v2`)

- `ExactClassicalProblem::build(domain, problem, max_ground_actions)` —
  grounded, capability-profiled problem construction
  (`ExactClassicalCapabilityProfile`, `LossyLowering` disclosed as lossy).
- `find_plan(max_depth, max_states)` / `find_label_plan(...)` return a
  `Pddl8Tape` or `ExactClassicalError`; bounds constants
  `EXACT_MAX_GROUND_ACTIONS`, `EXACT_MAX_PLAN_DEPTH`,
  `EXACT_MAX_SEARCH_STATES` cap the search.

## Search rails (`search`)

Portfolio of exact and exploit rails behind one scheduler:

- `ExactBfsRail` — breadth-first exact search over a `GroundProblem`.
- `QLensRail` — Q-lens-scored rail; `FairRailScheduler::select()` returns a
  `RailSelection`, with `ticks_since_exact()` exposing exact-rail starvation.
- `MfwPortfolio::solve()` — mass-function-weighted portfolio producing a
  `PortfolioOutcome`.
- `schedule_analysis`: `analyze_schedule` /
  `analyze_schedule_instrumented` return `ScheduleAnalysis64` with per-substage
  `AnalysisSubstageNs` timing and `CapacityDelta`s.

## Consequence and residualization (`consequence`)

- `PlanningResult` — final state, goal, step count, makespan.
- Horizons: `GoalReachabilityHorizon`, `MinimumMakespanHorizon`,
  `MakespanObservation` — horizon objects queried through a shared
  `horizon()` interface.
- `StandingConsequenceCache` — digest-keyed (`ExactStateKey`) cache with
  `lookup(state_digest, theory_digest)` and
  `admit(state_digest, theory_digest, result)`; `plan_with_standing_cache`
  wraps a planning run with the cache. Residual obligations are carried as
  `ResidualDecision` / `ResidualObligation` on `Residualizer`.

## Temporal grounding and validation

- `ground`: `GroundTemporalProblem`, `GroundDurativeAction`,
  `GroundDerivedPredicate` with `TypeIndex` / `QuantifierDomain`;
  trajectory constraints via `TrajectoryPolicy` and
  `ConstraintMonitor`/`MonitorFactory` (`MonitorState` machines).
- `validate`: `validate_plan` and `validate_temporal_plan_shape` return
  `PlanViolation` / `TemporalShapeViolation` lists.
- `execute`: `execute_tape`, `compute_plan_chain`,
  `execute_temporal_plan_instrumented` (per-substage `SubstageNs` timing).
- `parse`: `domain_from_pddl` / `problem_from_pddl` (PDDL8) and
  `domain31_from_pddl` / `problem31_from_pddl` (PDDL 3.1).
- `logical_time::LogicalTime` — tick-based logical clock used by temporal
  execution.

## Workflow execution surface (`workflow_cmd`)

The CLI-facing workflow layer: `dispatch`, `application`,
`observation_goal` and sibling submodules expose the workflow commands the
bcinr binary dispatches (workflow planning, observation/goal extraction,
application execution). Cognitive-side entry points:
`plan_exact_cognitive_workflow` (plus `_bounded` and `_hierarchical`
variants), the `CognitivePddlRuntime` execution stack in `downstream`
(`execute_cognitive_pddl`, `CognitivePddlExecutionSummary`), and the
production pipeline in `production` (`execute_pddl_to_powl`,
`ProductionMfwPlanner`, `PddlPowlStateReceipt`).

## Embedded workflows and MFW measures

- `embedded`: `TypedWorkflowPlan` / `VerifiedWorkflowPlan` /
  `WorkflowBatch` — typed, verified workflow plans with
  `ActionInvocation`s; errors are `EmbeddedWorkflowError` /
  `ActionLabelError`.
- `mfw`: mass-function measures — `MassVector`, `PositiveDistribution` /
  `WeightedDistribution`, `FrontierBoxes` / `FrontierMeasure`, `q_lens`
  scoring (`QValue`, `QLensError`).

## PDDL 3.1 / PDDL8 document model (`wasm4pm_compat`)

The `wasm4pm_compat` module defines the interchange AST shared with
wasm4pm: `Pddl31Domain` / `Pddl31Problem`, `Pddl31Action`,
`DurativeAction`, `DurationConstraint`, `TimedLiteral`,
`TrajectoryConstraint`, `Metric`/`MetricExpr`, `NumericExpr`/`NumericEffect`,
and the PDDL8 execution surface (`Pddl8Tape`, `Pddl8GroundAction`,
`Pddl8ExecutionLog`, `Pddl8ExecutionReceipt`, `Pddl8StepResult`,
`TemporalPlan`, `TemporalExecutionReceipt`). Bound constants:
`PDDL8_MAX_ARITY`, `PDDL8_MAX_PARAMS`, `PDDL8_MAX_CONJUNCTS`,
`PDDL8_MAX_GROUND`, `PDDL8_MAX_PLAN_DEPTH`.

## Concurrency, causal analysis, and the DFCM crown suite

- `concurrency::PddlConcurrencyAnalyzer` /
  `causal::PddlCausalAnalyzer` (and `causal_v2::PddlCausalAnalyzerV2`):
  post-execution analysis passes over executed plans
  (`ConcurrencyAnalysisError`, `CausalAnalysisError`).
- `dfcm_crown::run_dfcm_crown_suite` — the DFCM crown benchmark suite
  (`DfcmBenchReceipt`).
- `resource_ledger`: `ResourceLedger` / `ResourceLease` with typed
  `ResourceRefusal`s (`ResourceMode`, `ResourceRef`).
- `task`: `PddlTask` / `OwnedPddlTask` with `execute_cognitive_task` — the
  task-surface entry point consumed by the agent harness.
