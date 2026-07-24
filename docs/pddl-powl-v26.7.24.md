# PDDL → POWL production boundary — v26.7.24

## Status vocabulary

The release exposes two distinct semantic standings. They are never collapsed.

| Standing | Admitted PDDL | POWL projection | Execution evidence |
|---|---|---|---|
| `WitnessedConcurrentStrips` | Exact STRIPS plus typing | Reduced causal partial order with explicit independence and conflict witnesses | POWL v2 receipt plus tick-by-tick PDDL state replay and goal receipt |
| `ExactSequentialClassical` | Exact bounded classical conditions, equality, quantifiers, conditional effects, and numeric fluents/effects | Sequential POWL 2.0; no concurrency invented for rich semantics | POWL v2 receipt plus deterministic exact-planner replay and a semantic root binding theory, plan, choices, tape, and execution |

Temporal actions, timed initial literals, PDDL+ processes/events, derived predicates, trajectory constraints, preferences, metrics, continuous effects, and object-valued fluents are typed refusals on the current production rails.

## Primary embedded application API

Enable the `mfw-planner` feature and use the narrow prelude.

```rust
use bcinr_pddl::prelude::*;

let mut workflow = EmbeddedWorkflow::new(domain_pddl);
let verified = workflow.plan_pddl(problem_pddl)?;
let typed: TypedWorkflowPlan<ApplicationCommand> =
    verified.bind::<ApplicationCommand>()?;

println!("standing: {:?}", typed.standing);
println!("receipt: {}", typed.execution_root);

for batch in typed.batches() {
    application_broker.admit(batch, &typed.execution_root)?;
}
```

`EmbeddedWorkflow` keeps the stable domain and cache-preserving planning runtime inside the Rust application. `VerifiedWorkflowPlan` is manufactured only after the selected semantic rail and POWL execution verify. `ActionInvocation` normalizes planner labels before `TryFrom<ActionInvocation>` or `map_actions` converts them to native commands.

Application state can implement `WorkflowProblem` directly. Common positive STRIPS/typing problems can use `StripsProblemBuilder` and its validated `PddlProblemDocument` output instead of hand-concatenating PDDL.

See [`embedded-planning-paradigm.md`](embedded-planning-paradigm.md) for the architectural rationale, phase change, application patterns, delivered utilities, and remaining roadmap.

## Lower-level downstream API

Applications that need direct execution objects can use:

```rust
use bcinr_pddl::prelude::*;

let task = PddlTask::new(domain_pddl, problem_pddl);
let execution = execute_cognitive_task(task)?;

execution.verify()?;
println!("standing: {:?}", execution.standing());
println!("receipt: {}", execution.execution_root());

for batch in execution.batches()? {
    if !batch.actions.is_empty() {
        dispatch_after_broker_admission(batch.actions);
    }
}
```

`execute_cognitive_pddl(domain, problem)` is the equivalent two-argument convenience function. `CognitivePddlRuntime` preserves the exact-match planning cache across calls. `OwnedPddlTask` is serializable for queues and connector boundaries.

## Routing law

The router attempts the witnessed-concurrent rail first. It falls back to exact sequential classical planning only when the first rail returns a typed `PlannerFailure::Unsupported` admission result.

Parse failures, inconsistent inputs, search exhaustion, bound hits, validation defects, projection failures, deadlocks, and receipt mismatches are returned directly. They are never converted into fallback success.

## Receipt law

For witnessed concurrency, the downstream execution root is the PDDL state receipt. It binds:

- the initial state;
- each simultaneous POWL firing batch;
- every resolved ground action;
- before/after state roots;
- the POWL execution chain;
- the final state and goal.

Every batch checks all preconditions against one immutable pre-state. Delete/precondition and delete/add interference are refused before aggregate effects commit. The final parallel state must equal the validated sequential plan state.

For exact sequential classical planning, the downstream semantic root binds:

- admitted `theory_digest`;
- exact selected action labels;
- selected POWL 2.0 choices;
- compiled tape digest;
- POWL execution chain.

`verify()` reruns the exact bounded planner from the original PDDL source and requires identical theory, plan, tape, and execution receipt.

## Actuation boundary

These APIs manufacture and verify execution artifacts. They do not perform external side effects. A downstream system must submit the execution root and desired typed batch to its own admission broker and include the broker outcome in its actuation receipt.

## Local verifier

Run the focused release rail from the repository root:

```bash
bash scripts/verify-pddl-powl-v26.7.24.sh
```

The script checks the POWL 2.0 compiler/scheduler, receipt replay, exact classical PDDL rail, concurrent PDDL state execution, embedded application facade, deterministic problem builder, external-consumer APIs, all feature-gated targets, and both runnable examples. Its final successful line is:

```text
PDDL_TO_POWL_V26_7_24=ALIVE
```
