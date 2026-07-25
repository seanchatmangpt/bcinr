# Embedded Planning and Workflow: Control Logic Becomes an Internal Rust Capability

## Executive proposition

Most software treats planning and workflow as infrastructure outside the application.

A service sends state to a workflow server, BPMN engine, rules service, planner daemon, scheduler, or orchestration SaaS. That external system owns the process definition and execution cursor. It decides what should happen next, then invokes application workers through jobs, queues, webhooks, activities, or RPC.

`bcinr-pddl` establishes a different architecture:

> A Rust application can carry its own bounded planning and workflow capability, manufacture a valid course of action from current state and a goal, preserve the resulting process geometry in POWL, verify the execution evidence, and bind admitted actions to native Rust commands before any side effect occurs.

Planning is no longer merely a remote service consulted by the application. It becomes an internal computational faculty of the application.

This is not just a workflow engine moved into the same process. It changes the unit of control-flow construction.

Traditional code executes a path that a developer previously wrote. An embedded planning application defines an action theory, observes the current situation, states a goal, and manufactures the path required for that situation. POWL preserves ordering, lawful concurrency, choices, and execution evidence. Rust receives typed work only after the result has earned semantic standing.

That is a phase change from **executing predefined control flow** to **manufacturing verified control flow inside the application**.

---

## The conventional external-control model

A conventional workflow architecture often looks like this:

```text
Rust application
    │
    ├── serialize state or emit an event
    ▼
external workflow or planning system
    │
    ├── own workflow definition
    ├── own execution cursor
    ├── schedule activities
    └── call application workers
            │
            ▼
       side effects
```

This architecture can be appropriate for durable, cross-organizational processes. It also creates a second control plane beside the application:

- application state and workflow state must remain synchronized;
- domain types become transport DTOs;
- every decision may cross a process or network boundary;
- deployment and versioning include another runtime;
- local tests require an emulator, server, container, or mocked orchestration API;
- workflow semantics are often weaker than the application's type system;
- dynamic planning is bolted onto a system optimized for predefined graphs;
- concurrency is encoded manually in diagrams, DAGs, or orchestration code;
- evidence usually proves that tasks were scheduled, not that planning semantics were preserved.

The process lives in the external engine. The Rust application becomes an activity host.

---

## The embedded-control model

The embedded model reverses that relationship:

```text
Rust domain state
    │
    ├── WorkflowProblem projection
    ├── or StripsProblemBuilder
    ▼
resident PDDL domain + bounded problem
    │
    ├── semantic admission
    ├── planning
    ├── causal analysis
    ├── POWL projection
    ├── guarded POWL execution
    └── receipt verification
            │
            ▼
VerifiedWorkflowPlan
    │
    ├── ActionInvocation parsing
    ├── typed Rust command binding
    └── admitted batches + execution root
            │
            ▼
application-owned broker / transaction / queue / actor / task group
```

The application owns:

- the stable action theory;
- the state projection;
- the planning runtime instance;
- the native command vocabulary;
- authorization and policy;
- transactions and durability;
- retries and compensation;
- actual side effects.

The planning library owns:

- bounded parsing and admission;
- semantic routing;
- plan manufacture;
- causal and concurrency analysis;
- POWL construction and execution;
- receipt generation and replay verification;
- typed refusals when standing cannot be earned.

This division is deliberate:

- **The planner decides.**
- **POWL preserves and executes process geometry.**
- **Receipts establish what was admitted and executed.**
- **The Rust application actuates through its own explicit boundary.**

The library does not call user handlers during planning. A verified plan is a value. The host can inspect it, persist it, approve it, serialize it, enqueue it, simulate it, compare it, or reject it before any external mutation occurs.

---

## The phase change

The architectural transition is not merely:

```text
external workflow engine → in-process workflow engine
```

It is:

| Conventional model | Embedded planning model |
|---|---|
| Prewritten process | Runtime-manufactured process |
| External control plane | Application-owned cognitive capability |
| Engine-owned cursor | Receipt-bearing application value |
| String activity names | Typed Rust commands |
| Manually declared parallelism | Semantically witnessed parallelism |
| Static graph plus parameters | State + goal + action theory |
| Network integration by default | Local library composition by default |
| Hidden scheduler state | Explicit batches and execution root |
| Generic engine failures | Typed semantic refusals |

The decisive change is that code no longer has to contain every path. Code contains the lawful action space from which a path can be manufactured.

### Before: code contains the answer

Ordinary imperative software accumulates control flow:

```rust
if payment_authorized && inventory_available {
    reserve_inventory();
    send_confirmation();
} else if payment_authorized && backorder_allowed {
    create_backorder();
    notify_delay();
} else {
    request_intervention();
}
```

As the domain grows, this becomes nested branching, state machines, policy tables, saga definitions, workflow diagrams, or orchestration code. Each new combination adds another case that a developer must anticipate and maintain.

### After: code contains the action theory

An embedded planning application defines:

- what facts may be observed;
- what actions are available;
- what each action requires;
- what each action changes;
- what goal must hold;
- what semantic features are admitted;
- what structural and search bounds apply;
- how admitted actions bind to native commands.

The application then manufactures the control flow required for the current state.

This resembles the relationship between SQL and a query optimizer. Application authors describe the desired result and available relational structure. The optimizer manufactures an execution plan. Embedded planning generalizes that pattern from data retrieval to state-changing courses of action.

The planner becomes an **action-space optimizer and proof-producing control-flow manufacturer inside the application**.

---

## The application-facing API

The embedded layer is designed around ordinary Rust composition rather than a remote-client abstraction.

### `EmbeddedWorkflow`

`EmbeddedWorkflow` is a domain-scoped runtime intended to live beside a service, actor, aggregate, game system, device controller, or workflow supervisor.

```rust
let mut fulfillment = EmbeddedWorkflow::new(FULFILLMENT_DOMAIN);
```

The stable PDDL domain remains resident. Repeated problem instances reuse the internal standing cache in `CognitivePddlRuntime`.

This is materially different from constructing a planner RPC client. The planning capability is part of the application's own execution image.

### `WorkflowProblem`

`WorkflowProblem` lets a Rust type project its current state into a PDDL problem document.

```rust
impl WorkflowProblem for Order {
    fn to_pddl_problem(&self) -> Cow<'_, str> {
        // Manufacture the admitted observation and desired goal.
    }
}
```

The application does not need to expose its entire internal model. It manufactures a bounded planning observation containing only the facts relevant to the decision.

```text
rich application state → admitted planning observation
```

The projection may borrow a static fixture, render a validated builder, or generate a richer PDDL document directly.

### `StripsProblemBuilder`

Common positive STRIPS and typing problems should not require ad hoc string concatenation. `StripsProblemBuilder` provides deterministic construction for:

- validated problem and domain symbols;
- untyped and typed objects;
- positive initial facts;
- positive conjunctive goals;
- canonical document rendering.

```rust
let mut problem = StripsProblemBuilder::new("order-42", "fulfillment")?;
problem
    .add_typed_object("order-42", "order")?
    .add_fact("paid", ["order-42"])?
    .add_goal("reserved", ["order-42"])?
    .add_goal("notified", ["order-42"])?;

let problem = problem.build()?;
let verified = workflow.plan(&problem)?;
```

The builder is intentionally narrow. It does not pretend that negative conditions, disjunction, quantifiers, conditional effects, numeric fluents, temporal constructs, or preferences are positive STRIPS atoms. Richer semantics remain available through the explicit full-document path and are routed to the appropriate semantic rail.

This is an important design rule: **ergonomics must not flatten meaning**.

### `VerifiedWorkflowPlan`

A plan cannot enter `VerifiedWorkflowPlan` merely because search found an action sequence.

Construction is private. The embedded runtime first verifies the selected semantic rail and its POWL execution evidence. The resulting value exposes:

- the semantic standing used for the problem;
- the receipt-bound execution root;
- ordered scheduler batches;
- parsed application action invocations;
- a portable execution summary;
- the underlying verified execution when lower-level access is required.

The type makes a useful distinction:

```text
candidate plan → verified workflow plan → application command binding
```

### `ActionInvocation`

Planner labels are normalized into an explicit application representation:

```rust
ActionInvocation {
    label: "reserve(order-42,warehouse-a)",
    name: "reserve",
    arguments: vec!["order-42", "warehouse-a"],
}
```

The parser accepts the label forms emitted by the planning rails and common connector representations:

- `reserve(order-42,warehouse-a)`;
- `(reserve order-42 warehouse-a)`;
- `reserve order-42 warehouse-a`;
- `reserve`.

Malformed labels produce `ActionLabelError`. Ambiguous strings do not cross into application dispatch silently.

### `TypedWorkflowPlan<A>`

The host converts generic planner invocations into its own command enum or DTO:

```rust
let commands: TypedWorkflowPlan<FulfillmentCommand> =
    verified.bind::<FulfillmentCommand>()?;
```

A typical binding uses `TryFrom<ActionInvocation>`:

```rust
impl TryFrom<ActionInvocation> for FulfillmentCommand {
    type Error = BindingError;

    fn try_from(action: ActionInvocation) -> Result<Self, Self::Error> {
        match (action.name.as_str(), action.arguments.as_slice()) {
            ("reserve", [order, warehouse]) => Ok(Self::Reserve {
                order: OrderId::parse(order)?,
                warehouse: WarehouseId::parse(warehouse)?,
            }),
            _ => Err(BindingError::UnknownAction(action.label)),
        }
    }
}
```

The compiler can now enforce exhaustiveness, identifier parsing, command payloads, ownership, and application invariants after the planning boundary.

`map_actions` is available when a closure is more convenient than a `TryFrom` implementation.

### `TypedWorkflowBatch<A>`

A typed batch preserves one admitted scheduler tick.

Actions inside a batch may execute concurrently only when the concurrent rail supplied the required independence witnesses. Batches retain their admitted order.

The host decides what a batch means operationally:

- one database transaction;
- one outbox commit;
- a Tokio task group;
- parallel actor messages;
- commands submitted to a durable queue;
- an ECS command buffer;
- a device-control command set;
- a human approval packet.

The library exposes concurrency as evidence-bearing process structure, not as an instruction to spawn threads blindly.

---

## Why embedding planning is valuable

### 1. Planning becomes ordinary Rust composition

The runtime can be stored in a struct, passed by ownership, protected by a mutex, placed behind an actor, wrapped in a service layer, or instantiated per tenant.

The result is a normal Rust value. It composes with:

- enums and `TryFrom`;
- iterators;
- channels;
- transactions;
- async runtimes;
- actor systems;
- ECS architectures;
- command buses;
- outbox patterns;
- WASM hosts;
- property tests and fuzzers.

The application does not have to reshape itself around the lifecycle of an external workflow product.

### 2. Domain state does not cross an orchestration boundary by default

The facts needed for a decision can remain in-process. This reduces serialization, RPC, authentication, schema coordination, latency, and distributed failure surfaces.

For private, local, edge, desktop, game, industrial, and embedded applications, this may eliminate an entire service category.

### 3. Dynamic control flow stops being exceptional

Most workflow systems are optimized for predefined graphs. Dynamic planning is commonly treated as a specialist subsystem that returns a sequence to a workflow engine.

Here, planning and workflow are one pipeline:

```text
state + goal
    → semantic admission
    → selected actions
    → causal partial order
    → POWL process geometry
    → guarded execution
    → verified application work
```

The workflow is not authored first and populated later. It is manufactured from the current problem.

### 4. Concurrency is discovered from semantics

Developers usually decide which tasks may run in parallel, then encode that choice in futures, DAGs, BPMN gateways, job dependencies, or orchestration code.

The witnessed-concurrent rail instead proves pairwise independence for its admitted STRIPS/typing scope, projects the causal order into POWL, and exposes scheduler batches.

Concurrency changes from a manual optimization into a derived property of the action model.

The host still chooses the runtime mechanism. It no longer has to invent the dependency graph.

### 5. Richer semantics do not silently degrade

When a problem requires negative conditions, disjunction, equality, quantifiers, conditional effects, or numeric fluents, the router can select the exact bounded classical rail and conservatively project the selected plan sequentially.

The system does not claim concurrent standing without an appropriate independence proof.

Unsupported temporal, hybrid, derived, trajectory, preference, metric, continuous, and object-fluent surfaces remain typed refusals.

Application code can distinguish:

- unsupported capability;
- parse failure;
- inconsistent input;
- search exhaustion;
- bound exhaustion;
- projection failure;
- execution deadlock;
- receipt mismatch;
- malformed action label;
- application command-binding failure.

Broad syntax acceptance is less valuable than explicit semantics. A refusal is preferable to a plan whose meaning changed during translation.

### 6. The plan becomes a first-class application artifact

A verified plan can be:

- hashed and referenced by downstream receipts;
- persisted before actuation;
- attached to an audit record;
- presented for approval;
- replayed in a simulator;
- compared against a later replan;
- queued across process boundaries;
- inspected in tests;
- used to explain why a command exists.

External workflow systems often hide the execution plan behind engine state. The embedded model makes it an explicit value in the host language.

### 7. Deployment becomes simpler and more local

There is no mandatory planner daemon, workflow cluster, control-plane database, or orchestration network dependency for the planning decision.

That changes:

- cold-start latency;
- offline operation;
- edge deployment;
- single-binary distribution;
- local-first applications;
- test isolation;
- disaster recovery;
- tenant isolation;
- regulated-data containment;
- deterministic build and release practices.

An external durable executor may still be the correct actuator. It no longer has to own reasoning and process construction.

### 8. Testing moves from mocks to semantic fixtures

A developer can create a Rust value, project it into a problem, plan it, and inspect typed batches in one test process.

Tests can assert:

- which semantic standing was earned;
- whether actions were parallelized;
- which commands were manufactured;
- whether every planner action has a typed application binding;
- whether the execution root is stable;
- whether tampering is refused;
- whether changed state causes a different plan;
- whether a problem exceeds declared bounds.

This is stronger than mocking an orchestration client and asserting that an activity name was submitted.

### 9. Control-flow versioning can align with application versioning

An external workflow definition is often deployed and migrated separately from worker code. Embedded domains, bindings, tests, and release receipts can move in the same repository and artifact graph as the application code that interprets them.

This reduces semantic skew between:

- the action theory;
- the command enum;
- the handler implementation;
- the database schema;
- the release version.

### 10. Vendor independence becomes architectural rather than contractual

The application is not defined as a collection of callbacks owned by a particular orchestration product. It owns its decision artifacts and native command boundary.

Durable queues, workflow services, databases, and cloud schedulers can still be used as actuators or persistence mechanisms. They become replaceable projections below the planning boundary rather than the source of process meaning.

---

## Why Rust is a particularly strong host

Embedded planning benefits from language-level properties that Rust already treats as normal:

### Ownership makes the boundary explicit

A verified plan can be moved into a queue publisher, borrowed for inspection, or consumed into typed batches. Application code can make accidental reuse or mutation difficult by construction.

### Enums turn planner vocabulary into closed command sets

`TryFrom<ActionInvocation>` creates an explicit bridge from open textual planning labels to closed application types. Unknown actions are refusals, not dynamic handler lookups.

### Traits support domain-local projection

`WorkflowProblem` allows each aggregate, request, or projection to define its own bounded planning observation without forcing a global serialization model.

### Deterministic bounded search fits systems software

Ground-action, plan-depth, search-state, and tape bounds are visible application concerns. They can be configured, tested, and released like other resource limits.

### No mandatory runtime is imposed

The core API does not require Tokio, an actor framework, a database, or a queue. Adapters can remain optional and application-specific.

### The same capability can target server, desktop, WASM, game, and edge contexts

Moving planning into a library opens environments where operating a workflow control plane would be disproportionate or impossible.

---

## The safety and authority boundary

Internal planning must not become hidden internal actuation.

The embedded API deliberately stops at typed work. It does not register callbacks and invoke them while search or POWL execution is occurring.

This preserves six invariants:

1. **Planning remains inspectable.** The admitted observation and resulting process can be replayed and verified.
2. **Application authorization remains authoritative.** A semantically valid command can still be refused by tenancy, identity, policy, or current-state checks.
3. **Transactions remain application-owned.** The host determines atomicity and durability.
4. **Retries remain explicit.** A planner result is not confused with a completed external effect.
5. **Receipts can be chained.** The host can bind actuation evidence to `execution_root`.
6. **Replanning remains possible.** If observations change before actuation, the host can discard the stale plan and manufacture a new one.

The correct relationship is:

```text
verified plan ≠ completed business operation
```

A verified plan has standing as a decision artifact. The application gives standing to actuation through its own broker, observation, and receipts.

---

## Application patterns

### Transactional service

A service projects a database snapshot into a problem, receives typed batches, and writes the first batch plus the planning root into an outbox transaction. Workers actuate commands and append effect receipts.

### Actor or aggregate

An actor owns `EmbeddedWorkflow`. Each message changes local state, the actor replans, and typed batches become messages to child actors or supervised workers.

### Async Rust service

A Tokio service maps one admitted batch to a `JoinSet` or task group. It awaits completion before advancing to the next tick. The POWL batch boundary defines lawful concurrency; the service defines runtime scheduling and failure policy.

### Game and simulation system

An ECS system projects world state into a bounded problem. The plan becomes typed component commands or job batches. Planning remains local and can be tied to deterministic simulation receipts.

### Edge and device software

A device carries the domain and planner in its deployed binary. It manufactures actions without cloud availability while exporting compact roots and receipts for later synchronization.

### Human-in-the-loop operations

A verified plan becomes an approval packet. A user approves a batch, changes observations, or rejects an action. The application can replan without requiring a separately deployed workflow model.

### Build and release automation

Repository state, dependency state, test receipts, and release goals become the problem. Typed commands bind to build steps, checks, publication proposals, or deployment requests. The actuation broker retains exclusive authority over the filesystem, network, registry, and cloud APIs.

### Case management and enterprise operations

Eligibility facts, required documents, approvals, deadlines, and completion goals can generate a process specific to each case instead of forcing every case through one universal static diagram.

---

## What has been delivered

The current application utility layer includes:

- `EmbeddedWorkflow` for domain-resident, cache-preserving planning;
- `WorkflowProblem` for Rust-state projection;
- `StripsProblemBuilder` for deterministic common problem construction;
- `PddlProblemDocument` as a validated builder output;
- `ActionInvocation` for normalized action labels and arguments;
- `VerifiedWorkflowPlan` as the receipt-verified application boundary;
- `TypedWorkflowPlan<A>` and `TypedWorkflowBatch<A>` for native command binding;
- `bind` through `TryFrom<ActionInvocation>`;
- `map_actions` for closure-based binding;
- a narrow `bcinr_pddl::prelude::*`;
- an external-consumer integration test;
- a runnable `embedded_workflow` example;
- focused release-verifier coverage for the complete surface.

The existing lower-level API remains available for applications that need direct access to `CognitivePddlRuntime`, `CognitivePddlExecution`, POWL batches, summaries, or execution roots.

---

## Remaining gaps and next abstraction layers

The current facade closes the largest usability gap: internal planning output can become verified, typed, application-ready work. The following layers would deepen the paradigm.

### 1. Rich typed problem construction

`StripsProblemBuilder` intentionally covers only positive STRIPS and typing. A richer builder should expose explicit constructors for:

- negative and disjunctive conditions;
- equality;
- existential and universal conditions;
- conditional and quantified effects;
- numeric fluents and effects.

It must preserve the condition/effect tree and route to exact semantics. It must never flatten rich expressions into a simpler representation.

### 2. Domain construction and compile-time binding checks

Stable domains are still supplied as PDDL documents. A future domain DSL or macro could provide:

- validated predicates and parameter types;
- action preconditions and effects;
- deterministic domain rendering;
- generated Rust command enums;
- compile-time or test-time proof that every action has a binding;
- domain digest constants.

The macro should generate artifacts from one admitted model rather than creating a second hand-maintained schema.

### 3. Domain registries and version identity

Long-lived applications need stable identity for embedded action theories:

- domain digest;
- semantic version;
- command-binding version;
- compatibility checks for persisted plans;
- migration policy;
- multi-domain routing inside one process.

A persisted plan should state exactly which action theory and command vocabulary produced it.

### 4. Receipt-bearing actuation broker

The host currently receives `execution_root` and owns actuation. A companion abstraction should standardize—but not hide—the next boundary:

```text
planning execution root
    → command dispatch receipt
    → observed effect receipt
```

It should support transactions, outbox persistence, queues, actors, and async task groups without making one runtime mandatory.

### 5. Durable cursor and resumability

`TypedWorkflowPlan` exposes ordered batches. Durable applications need an explicit cursor that records:

- plan root;
- next tick;
- completed command receipts;
- partial-batch policy;
- retry generation;
- superseding replan root.

The cursor should remain application data, not hidden engine state.

### 6. Observation-driven residual replanning

Applications change between decision and actuation. A replan utility should compare:

- original admitted observation;
- new observation;
- completed effects;
- residual goal;
- still-valid plan suffix;
- replacement plan.

This turns exception handling from ad hoc branching into residual planning.

### 7. Async, actor, outbox, and ECS adapters

Small optional adapters can make the typed batch boundary natural for:

- Tokio task groups;
- actor mailboxes;
- command buses;
- transactional outboxes;
- durable queue publishers;
- ECS command buffers.

The core planning library should not acquire a mandatory async runtime.

### 8. Policy and authorization composition

A plan may be semantically valid but institutionally forbidden. The application boundary should support policy checks over typed commands with refusal receipts that distinguish:

- planner capability refusal;
- application binding refusal;
- authorization refusal;
- stale-observation refusal;
- actuation failure.

### 9. Optimization, preferences, resources, and time

The current exact rail is bounded classical plan finding, not a general optimizing or temporal planner. Cost, preferences, metrics, duration, resources, and continuous change need explicit semantic rails rather than being smuggled into action ordering.

### 10. Rich exact-state summaries

The witnessed concurrent rail exposes its final propositional state. The exact richer-classical rail currently exposes deterministic semantic replay and a semantic root rather than a portable rich numeric state. A canonical exact-state summary would improve downstream inspection and receipt chaining.

### 11. WASM and constrained deployment profiles

The internal model is especially valuable for local and edge software. Future profiles should make memory, action count, search depth, receipt size, and supported semantics explicit for WASM and constrained devices.

---

## Adoption ladder

The paradigm can be adopted incrementally.

### Stage 1: decision support

Plan and inspect typed commands while existing code remains authoritative over which path is used.

### Stage 2: command manufacture

Use the planner to create command batches while an existing service, workflow system, or queue remains the durable executor.

### Stage 3: residual replanning

After each observed effect, update state and replan the remaining goal.

### Stage 4: receipt-linked actuation

Bind command and effect receipts to the planning execution root.

### Stage 5: planning-native application architecture

Represent major operational capabilities as action theories. Services, actors, devices, or aggregates manufacture control flow from admitted observations and goals rather than accumulating hand-authored case trees.

The application has not outsourced its business logic. It has moved from enumerating paths to defining the lawful space from which paths can be manufactured.

---

## Paradigm statement

The phase change can be stated precisely:

> Workflow ceases to be a remote diagram or predefined state machine that drives a Rust application. It becomes a verified process artifact manufactured inside the Rust application from current state, available actions, and a goal.

The result is not an instruction string sent blindly to a handler. It is a receipt-bound, semantically classified, partially ordered set of actions that can be converted into native Rust commands and admitted by the application's own actuation boundary.

This makes planning a general application primitive:

```text
observe → bound → plan → construct process → verify → bind → actuate → receipt
```

Once planning and workflow become internal, they can be used anywhere ordinary control flow is used today—but with runtime synthesis, explicit semantic standing, lawful concurrency, typed refusals, and replayable evidence.

That is the new paradigm.
