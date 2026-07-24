# Embedded Planning in Rust: Developer Quickstart

This guide shows how to use `bcinr-pddl` as an internal planning and workflow capability inside a Rust application.

The core boundary is:

```text
Rust state
    → bounded PDDL problem
    → planning and semantic admission
    → POWL execution and verification
    → typed Rust command batches
    → application-owned actuation broker
```

The planner manufactures and verifies work. It does not call application handlers or perform external side effects.

## Enable the application surface

```toml
[dependencies]
bcinr-pddl = { version = "26.7.24", features = ["mfw-planner"] }
```

Import the narrow prelude:

```rust
use bcinr_pddl::prelude::*;
```

## 1. Install a resident domain

A domain defines the application actions, their preconditions, and their effects.

```rust
const FULFILLMENT_DOMAIN: &str = r#"
(define (domain fulfillment)
  (:requirements :strips)
  (:predicates
    (paid)
    (inventory-reserved)
    (customer-notified))

  (:action reserve-inventory
    :parameters ()
    :precondition (paid)
    :effect (inventory-reserved))

  (:action notify-customer
    :parameters ()
    :precondition (paid)
    :effect (customer-notified)))
"#;

let mut workflow = EmbeddedWorkflow::new(FULFILLMENT_DOMAIN)?;
```

Construction parses the domain immediately. Invalid syntax is refused before the runtime is installed.

The runtime exposes two useful identities:

```rust
assert_eq!(workflow.domain_name(), "fulfillment");
println!("domain source root: {}", workflow.domain_source_root());
```

`domain_source_root()` is the BLAKE3 root of the exact installed source. It is useful for configuration records, deployment manifests, logs, and application-level compatibility checks.

The semantic execution rails still perform their own receipt-producing admission when a problem is planned.

## 2. Plan a common STRIPS problem

For positive STRIPS and typing, use `plan_strips`. It automatically binds the problem to the installed domain name.

```rust
let verified = workflow.plan_strips("order-42", |problem| {
    problem
        .add_nullary_fact("paid")?
        .add_nullary_goal("inventory-reserved")?
        .add_nullary_goal("customer-notified")?;
    Ok(())
})?;
```

`StripsProblemBuilder` validates symbols, rejects conflicting object types, sorts objects/facts/goals, removes duplicates, and manufactures a canonical document. Equivalent insertion orders produce the same PDDL problem.

Typed objects and parameterized atoms are also supported:

```rust
let mut problem = workflow.strips_problem("shipment-7")?;
problem
    .add_typed_object("shipment-7", "shipment")?
    .add_typed_object("warehouse-a", "warehouse")?
    .add_fact("located-at", ["shipment-7", "warehouse-a"])?
    .add_goal("dispatched", ["shipment-7"])?;

let verified = workflow.plan(&problem.build()?)?;
```

The builder is intentionally limited to positive STRIPS/typing. It does not flatten negative conditions, disjunction, equality, quantifiers, conditional effects, numeric fluents, time, preferences, or metrics into a weaker representation.

## 3. Project native Rust state

For application-owned state, implement `WorkflowProblem`.

```rust
use std::borrow::Cow;

struct Order {
    id: u64,
    paid: bool,
}

impl WorkflowProblem for Order {
    fn to_pddl_problem(&self) -> Cow<'_, str> {
        let mut problem = StripsProblemBuilder::new(
            format!("order-{}", self.id),
            "fulfillment",
        )
        .expect("application identifiers must be valid PDDL symbols");

        if self.paid {
            problem
                .add_nullary_fact("paid")
                .expect("domain predicate is a source constant");
        }

        problem
            .add_nullary_goal("inventory-reserved")
            .expect("domain predicate is a source constant")
            .add_nullary_goal("customer-notified")
            .expect("domain predicate is a source constant");

        Cow::Owned(
            problem
                .build()
                .expect("application projection always supplies a goal")
                .into_string(),
        )
    }
}

let order = Order { id: 42, paid: true };
let verified = workflow.plan(&order)?;
```

`WorkflowProblem` is an observation boundary. It should expose only the state relevant to the planning decision, not serialize the entire application object graph.

## 4. Inspect the earned standing

Every verified workflow identifies the semantic standing used to manufacture it.

```rust
match verified.standing() {
    CognitiveExecutionStanding::WitnessedConcurrentStrips => {
        println!("concurrency is backed by STRIPS independence witnesses");
    }
    CognitiveExecutionStanding::ExactSequentialClassical => {
        println!("rich classical semantics were preserved sequentially");
    }
}

println!("planning execution root: {}", verified.execution_root());
```

The two standings are not interchangeable:

- `WitnessedConcurrentStrips` may expose multiple independent actions in one scheduler tick.
- `ExactSequentialClassical` preserves richer bounded classical semantics and does not invent concurrency without a corresponding proof.

`VerifiedWorkflowPlan` has a private constructor. The only normal way to obtain one is through an `EmbeddedWorkflow` after semantic and POWL verification succeeds.

## 5. Bind planner actions to native commands

Define the application's closed command vocabulary.

```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
enum FulfillmentCommand {
    ReserveInventory,
    NotifyCustomer,
}

impl TryFrom<ActionInvocation> for FulfillmentCommand {
    type Error = String;

    fn try_from(action: ActionInvocation) -> Result<Self, Self::Error> {
        match (action.name.as_str(), action.arguments.as_slice()) {
            ("reserve-inventory", []) => Ok(Self::ReserveInventory),
            ("notify-customer", []) => Ok(Self::NotifyCustomer),
            _ => Err(format!("unbound planner action: {}", action.label)),
        }
    }
}

let commands = verified
    .bind::<FulfillmentCommand>()
    .expect("every admitted action must have an application binding");
```

The compiler now participates in the workflow boundary. Application code can require typed identifiers, validate arguments, reject unknown actions, and keep command handling exhaustive.

When a `TryFrom` implementation is inconvenient, use `map_actions`:

```rust
let commands = verified.map_actions(|action| {
    registry.bind(action)
})?;
```

Binding is pure. It does not execute commands.

## 6. Consume admitted batches

```rust
println!("standing: {:?}", commands.standing());
println!("execution root: {}", commands.execution_root());

for batch in commands.batches() {
    println!("tick {}", batch.tick());
    println!("parallel: {}", batch.is_parallel());

    for command in batch.actions() {
        println!("  {command:?}");
    }
}
```

Actions in one batch may be submitted concurrently only because the selected rail admitted them in the same POWL scheduler tick. Ordered batches must retain their tick order.

The application chooses the operational interpretation:

- database transaction;
- transactional outbox;
- Tokio task group;
- actor messages;
- durable queue publications;
- ECS command buffer;
- device commands;
- human approval packet.

## 7. Keep actuation application-owned

A broker should receive both the typed batch and the planning execution root.

```rust
trait CommandBroker<A> {
    type Error;

    fn admit(
        &mut self,
        execution_root: &str,
        batch: &TypedWorkflowBatch<A>,
    ) -> Result<(), Self::Error>;
}

for batch in commands.batches() {
    broker.admit(commands.execution_root(), batch)?;
}
```

The broker remains responsible for:

- authorization;
- tenancy;
- current-state checks;
- idempotency;
- transactions;
- durability;
- retries;
- compensation;
- effect observation;
- actuation receipts.

A verified plan is a decision artifact, not evidence that an external effect occurred.

```text
verified workflow plan ≠ completed business operation
```

## 8. Use rich classical PDDL explicitly

When the problem needs richer conditions or effects, provide the complete PDDL document through `WorkflowProblem` or `plan_pddl`.

```rust
let verified = workflow.plan_pddl(r#"
(define (problem finish-all)
  (:domain rich-domain)
  (:objects a b - item)
  (:init (ready a) (ready b))
  (:goal (forall (?x - item) (done ?x))))
"#)?;
```

The automatic router attempts the witnessed concurrent rail first. It falls back to exact bounded classical planning only after a typed unsupported-capability result from the first rail.

Parse failures, inconsistent input, search exhaustion, bound exhaustion, projection defects, deadlocks, and receipt mismatches are not converted into fallback success.

## 9. Configure exact-search bounds

```rust
let config = CognitivePddlConfig {
    concurrent: PddlPowlConfig::default(),
    exact: ExactCognitiveBounds {
        max_ground_actions: 20_000,
        max_plan_depth: 32,
        max_search_states: 250_000,
    },
};

let mut workflow = EmbeddedWorkflow::with_config(
    FULFILLMENT_DOMAIN,
    config,
)?;
```

Bounds are application policy. Treat them like memory, latency, queue, and transaction limits: configure them explicitly, test them, and include them in release evidence.

## 10. Handle errors by boundary

Domain installation can fail with `Pddl8Error`:

```rust
let workflow = EmbeddedWorkflow::new(domain_pddl)?;
```

Planning and application preparation use `EmbeddedWorkflowError`:

```rust
match workflow.plan(&state) {
    Ok(verified) => consume(verified),
    Err(EmbeddedWorkflowError::ProblemBuild(error)) => {
        report_invalid_application_projection(error);
    }
    Err(EmbeddedWorkflowError::Planning(error)) => {
        handle_semantic_or_search_failure(error);
    }
    Err(EmbeddedWorkflowError::InvalidActionLabel(error)) => {
        refuse_application_binding_boundary(error);
    }
}
```

Native command binding has the error type chosen by the application through `TryFrom` or `map_actions`.

Do not collapse these boundaries into one generic retry. A malformed problem, unsupported semantic feature, exhausted search bound, receipt mismatch, and missing command binding require different responses.

## 11. Cross-process trust rule

`WorkflowBatch`, `TypedWorkflowPlan`, and `TypedWorkflowBatch` can be serialized for logging, proposals, queue payloads, or inspection. They are intentionally not generically deserializable as verified standing-bearing values.

A JSON object with the same fields does not prove that:

- the domain was admitted;
- the plan was valid;
- POWL executed successfully;
- the receipt root is authentic;
- the command binding corresponds to the admitted actions.

Across a process or trust boundary, treat serialized typed work as an untrusted proposal unless the receiver re-establishes standing through one of these patterns:

1. replan and verify from the admitted domain/problem sources;
2. verify a future durable replay envelope carrying the complete required evidence;
3. verify an application-owned signed envelope that binds source identity, execution root, commands, and authorization policy.

Serialization preserves data. It does not manufacture standing.

## 12. Cache-preserving lower-level use

Applications that need direct execution objects can use `CognitivePddlRuntime`:

```rust
let mut runtime = CognitivePddlRuntime::default();
let execution = runtime.execute(domain_pddl, problem_pddl)?;
execution.verify()?;

let summary = execution.summary()?;
println!("{}", summary.to_pretty_json()?);
```

`OwnedPddlTask` remains useful as an untrusted transport DTO for queues and connectors. Receiving code must still execute and verify the task.

## Complete mental model

```text
EmbeddedWorkflow
    resident domain source
    domain name
    domain source root
    cache-preserving runtime
            │
            ▼
WorkflowProblem / StripsProblemBuilder
    bounded observation
    explicit goal
            │
            ▼
semantic router
    witnessed concurrent STRIPS
    or exact sequential classical
            │
            ▼
POWL execution + receipt verification
            │
            ▼
VerifiedWorkflowPlan
            │
            ▼
ActionInvocation → native command type
            │
            ▼
TypedWorkflowPlan
            │
            ▼
application-owned broker and actuation receipts
```

The architectural rule is simple:

> Put planning and workflow manufacture inside the Rust application. Keep authority over external mutation at an explicit application boundary.
