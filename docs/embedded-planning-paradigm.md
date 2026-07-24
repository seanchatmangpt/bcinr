# Embedded Planning and Workflow: Control Logic Becomes an Internal Rust Capability

## Thesis

Most software treats **workflow** and **planning** as infrastructure outside the application.

The application sends state to a workflow server, BPMN engine, rules service, planner daemon, scheduler, or orchestration SaaS. That external system decides what should happen next, then calls back into the application through jobs, queues, webhooks, activities, or RPC.

`bcinr-pddl` introduces a different architecture:

> A Rust application can carry its own bounded planning and workflow capability, manufacture a valid course of action from its current state and goal, execute the resulting process geometry internally, verify the execution receipt, and then bind the admitted actions to native Rust commands.

Planning is no longer an external service consulted by the application. Planning becomes an internal computational faculty of the application.

This is not merely an embedded version of a conventional workflow engine. It changes the unit of control-flow construction.

Traditional application code chooses a path that a developer previously wrote. An embedded planning application describes the state, the available actions, and the desired condition, then manufactures the path at runtime under explicit semantic bounds. POWL preserves the resulting partial order, lawful concurrency, choices, and execution evidence. Rust receives typed work only after that result has earned standing.

That is a phase change from **executing predefined control flow** to **manufacturing verified control flow inside the application**.

---

## The conventional boundary

A conventional workflow architecture usually looks like this:

```text
Rust application
    │
    ├── serialize state or emit event
    │
    ▼
external workflow/planning system
    │
    ├── own workflow definition
    ├── own execution cursor
    ├── schedule activities
    └── call application workers
            │
            ▼
       side effects
```

This model can be useful, but it creates a second control plane beside the application:

- application state and workflow state must remain synchronized;
- domain types become transport DTOs;
- every decision crosses a process or network boundary;
- deployment and versioning include another runtime;
- workflow semantics are often weaker than the application's type system;
- local tests require an emulator, server, container, or mocked orchestration API;
- dynamic planning is frequently bolted onto a static workflow model;
- concurrency is encoded manually in diagrams or orchestration code;
- evidence usually proves that tasks were scheduled, not that the admitted planning semantics were preserved.

The external engine becomes the place where the process lives. The Rust application becomes an activity host.

---

## The embedded boundary

The embedded model reverses that relationship:

```text
Rust domain state
    │
    ├── WorkflowProblem projection
    ▼
resident PDDL domain + bounded problem
    │
    ├── semantic admission
    ├── planning
    ├── POWL projection
    ├── guarded execution
    └── receipt verification
            │
            ▼
VerifiedWorkflowPlan
    │
    ├── ActionInvocation
    ├── typed Rust command binding
    └── admitted batches + execution root
            │
            ▼
application-owned broker / transaction / queue / actor / task group
```

The application owns the domain, the runtime, the state projection, the command types, and the actuation policy. The planning library owns only the bounded manufacture and verification of candidate control flow.

This separation is critical:

- **The planner decides.**
- **POWL preserves and executes process geometry.**
- **Receipts establish what was admitted and executed.**
- **The Rust application actuates through its own explicit boundary.**

The library does not call user handlers during planning. A verified plan is a value. The host may serialize it, inspect it, persist it, enqueue it, approve it, simulate it, transact it, or reject it before any side effect occurs.

---

## The application-facing abstractions

The embedded API adds six primary concepts.

### `EmbeddedWorkflow`

A domain-scoped runtime intended to live beside a Rust service, actor, aggregate, game system, device controller, or workflow supervisor.

The stable PDDL domain is loaded once. Repeated problem instances reuse the internal standing cache.

```rust
let mut fulfillment = EmbeddedWorkflow::new(FULFILLMENT_DOMAIN);
```

This is materially different from creating an RPC client for a planner. The planning runtime is part of the application's own execution image.

### `WorkflowProblem`

A trait implemented by application state or request types.

```rust
impl WorkflowProblem for Order {
    fn to_pddl_problem(&self) -> Cow<'_, str> {
        // Project current Rust state and the desired condition into PDDL.
    }
}
```

The application does not need to expose its entire internal model. It manufactures a bounded observation containing only the facts relevant to the planning decision.

This creates a clean architectural seam:

```text
rich application state → admitted planning observation
```

The problem document can be borrowed, generated, cached, or assembled from a domain projection.

### `VerifiedWorkflowPlan`

A plan cannot enter this type merely because search found an action sequence.

Construction requires the selected semantic rail to verify its execution. The value carries:

- the semantic standing used for the problem;
- receipt-bound execution root;
- ordered scheduler batches;
- parsed application action invocations;
- access to the portable execution summary.

The constructor is private. Application code receives this value only through the embedded runtime.

### `ActionInvocation`

Planner labels are normalized into an explicit application representation:

```rust
ActionInvocation {
    label: "reserve(order-42,warehouse-a)",
    name: "reserve",
    arguments: vec!["order-42", "warehouse-a"],
}
```

The parser accepts function-style, S-expression, whitespace, and zero-argument labels. Malformed labels produce a typed refusal instead of entering application dispatch as ambiguous strings.

### `TypedWorkflowPlan<A>`

The host converts generic invocations into its own command enum or DTO:

```rust
let commands: TypedWorkflowPlan<FulfillmentCommand> =
    verified.bind::<FulfillmentCommand>()?;
```

This is the point where planning vocabulary becomes application vocabulary.

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

The compiler can now enforce exhaustiveness, domain identifiers, command payloads, and application invariants after the planning boundary.

### `TypedWorkflowBatch<A>`

A batch preserves one admitted scheduler tick.

Actions inside a batch may execute concurrently because the concurrent rail supplied the required independence witnesses. Batches retain their admitted order.

The host decides what a batch means operationally:

- one database transaction;
- one outbox commit;
- a set of Tokio tasks;
- parallel actor messages;
- commands submitted to a durable queue;
- a game-engine job group;
- a device-control command set;
- a human approval packet.

The library exposes concurrency as evidence-bearing process structure, not as an instruction to spawn threads blindly.

---

## Why this is valuable

### 1. Domain state no longer crosses an orchestration boundary by default

The state needed for a decision can remain in-process. This reduces serialization, RPC, authentication, schema coordination, and distributed failure surfaces.

For local, private, edge, desktop, game, industrial, and embedded applications, this may eliminate an entire service category.

### 2. Planning participates in ordinary Rust composition

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

### 3. Dynamic control flow stops being exceptional

Most workflow systems are optimized for predefined graphs. Dynamic planning is commonly treated as a specialist subsystem that returns a sequence to the workflow engine.

Here, planning and workflow are one pipeline:

```text
state + goal
    → admitted semantics
    → selected actions
    → causal partial order
    → POWL process geometry
    → guarded execution
    → verified application work
```

The workflow is not authored first and populated later. It is manufactured from the current problem.

### 4. Concurrency is discovered from semantics

Developers normally decide which tasks may run in parallel, then encode that choice in futures, DAGs, BPMN gateways, job dependencies, or orchestration code.

The witnessed-concurrent rail instead proves pairwise independence for the admitted STRIPS/typing scope, projects the causal order into POWL, and exposes scheduler batches.

This changes concurrency from a manual optimization into a derived property of the action model.

The host still chooses the execution mechanism, but it no longer has to invent the dependency graph.

### 5. Richer classical semantics do not silently degrade

When a problem requires negative conditions, disjunction, equality, quantifiers, conditional effects, or numeric fluents, the router can select the exact bounded classical rail and conservatively project the plan sequentially.

The system does not claim concurrent standing without an appropriate independence proof.

Unsupported temporal, hybrid, derived, trajectory, preference, metric, continuous, and object-fluent surfaces remain typed refusals.

This is more valuable than broad syntax acceptance with ambiguous semantics. Application code can distinguish:

- unsupported capability;
- parse failure;
- inconsistent input;
- search exhaustion;
- bound exhaustion;
- projection failure;
- execution deadlock;
- receipt mismatch;
- command-binding failure.

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

That has direct implications for:

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

An external durable executor may still be the correct actuator. The phase change is that it no longer has to be the owner of reasoning and process construction.

### 8. Testing moves from mocks to semantic fixtures

A developer can create a Rust value, project it into a problem, plan it, and inspect typed batches in one test process.

Tests can assert:

- which semantic standing was earned;
- whether actions were parallelized;
- which commands were manufactured;
- whether every planner action has a typed application binding;
- whether the execution root is stable;
- whether tampering is refused;
- whether a changed state causes a different plan;
- whether a problem exceeds declared bounds.

This is substantially stronger than mocking an orchestration client and asserting that an activity name was submitted.

---

## Why this is a new paradigm rather than a smaller workflow engine

An embedded database is not merely a remote database with the network removed. It changes what applications can assume about locality, transactions, packaging, and ownership.

Embedded planning has the same character.

The important transition is not:

```text
external workflow engine → in-process workflow engine
```

It is:

```text
prewritten process → runtime-manufactured process
external control plane → application-owned cognitive capability
opaque engine cursor → receipt-bearing application value
string activity names → typed Rust commands
manually declared parallelism → semantically witnessed parallelism
```

This changes the architecture of control itself.

### Before: code contains the answer

In ordinary imperative software, the developer encodes the decision tree:

```rust
if inventory_available && payment_authorized {
    reserve_inventory();
    send_confirmation();
} else if ...
```

As the domain grows, this becomes nested conditions, state machines, policy tables, workflow diagrams, saga definitions, or orchestration code. The control flow is a historical accumulation of cases anticipated by developers.

### After: code contains the action theory

In an embedded planning application, the developer defines:

- what facts may be observed;
- what actions exist;
- what each action requires;
- what each action changes;
- what goal must hold;
- what semantics and resource bounds are admitted;
- how admitted actions bind to native commands.

The application manufactures the control flow required for the present state.

This resembles the relationship between SQL and a query optimizer. Application authors describe the desired result and available relational structure; the optimizer manufactures an execution plan. Embedded planning generalizes that pattern from data retrieval to state-changing courses of action.

The planner is therefore an **action-space optimizer and proof-producing control-flow manufacturer inside the application**.

### A new software layer

The conventional stack is often described as:

```text
business logic
workflow/orchestration
workers and infrastructure
```

The embedded model introduces a new internal layer:

```text
application goals and observations
bounded action theory
planning + process manufacture
verified typed command batches
application actuation broker
```

This layer can sit inside a library, service, aggregate, actor, or device. It is smaller than a control plane but semantically richer than a hand-authored state machine.

---

## The safety and authority boundary

Internal planning must not become hidden internal actuation.

The embedded API deliberately stops at typed work. It does not register callbacks and call them while planning. That design preserves several invariants:

1. **Planning remains referentially inspectable.** The same admitted observation can be replayed and verified.
2. **Application authorization remains authoritative.** A command can still be rejected by policy, tenancy, identity, or current-state checks.
3. **Transactions remain application-owned.** The host determines atomicity and durability.
4. **Retries remain explicit.** A planner result is not confused with a completed side effect.
5. **Receipts can be chained.** The host can bind its actuation receipt to `execution_root`.
6. **Replanning remains possible.** If observations change before actuation, the host may discard the old plan and manufacture a new one.

The correct model is:

```text
verified plan ≠ completed business operation
```

A verified plan has standing as a decision artifact. The application gives standing to actuation through its own broker and receipts.

---

## Application patterns

### Transactional service

A service projects a database snapshot into a problem, receives typed batches, and writes the first batch plus the planning root into an outbox transaction. Workers actuate commands and append receipts.

### Actor or aggregate

An actor owns `EmbeddedWorkflow`. Each message changes local state, the actor replans, and typed batches become messages to child actors or supervised workers.

### Async Rust service

A Tokio service maps one admitted batch to a `JoinSet` or task group. It awaits completion before advancing to the next tick. The POWL batch boundary defines lawful concurrency; the service defines runtime scheduling and error policy.

### Game and simulation system

An ECS system projects world state into a bounded problem. The plan becomes typed component commands or job batches. Planning can remain deterministic and local to the simulation tick.

### Edge and device software

A device carries the domain and planner in the deployed binary. It can manufacture actions without cloud availability, while still exporting compact execution roots and receipts for later synchronization.

### Human-in-the-loop operations

A plan becomes an approval packet. A user approves a batch, modifies observations, or rejects an action. The system can replan without requiring a separately deployed workflow model.

### Build, release, and infrastructure automation

Repository state, dependency state, test receipts, and release goals become the problem. Typed commands bind to build steps, checks, publication requests, or deployment proposals. The actuation broker retains the authority to touch the filesystem, network, package registry, or cloud API.

---

## Gaps identified and the next utility layers

The current embedded facade closes the largest usability gap: it turns internal planning output into verified, typed, application-ready work. Several additional layers would deepen the paradigm.

### 1. Typed problem construction

`WorkflowProblem` currently gives applications complete control over PDDL projection. The next ergonomic layer should provide:

- a bounded `ProblemBuilder` for objects, facts, functions, and goals;
- validated PDDL symbols;
- typed atom constructors;
- deterministic canonical rendering;
- derive or macro support for common Rust state projections.

The builder must not flatten rich conditions. It should expose separate constructors for the exact semantics it can preserve.

### 2. Domain registries and version identity

Long-lived applications need stable identity for embedded action theories:

- domain digest;
- semantic version;
- migration policy;
- command-binding version;
- compatibility checks for persisted plans;
- multi-domain routing inside one process.

A verified plan should be able to state exactly which action theory and command vocabulary produced it.

### 3. Receipt-bearing actuation broker

The host currently receives `execution_root` and owns actuation. A future companion abstraction should standardize—but not hide—the next boundary:

```text
planning execution root
    → command dispatch receipt
    → effect observation receipt
```

This broker should support transactions, outbox persistence, queues, actors, and async task groups without making one runtime mandatory.

### 4. Durable cursor and resumability

`TypedWorkflowPlan` exposes ordered batches. Durable applications will benefit from a cursor that records:

- plan root;
- next tick;
- completed command receipts;
- partial batch policy;
- retry generation;
- superseding replan root.

The cursor should remain application data rather than hidden engine state.

### 5. Observation-driven replanning

Real applications change between decision and actuation. A replan utility should compare:

- original admitted observation;
- new observation;
- completed effects;
- residual goal;
- still-valid suffix;
- replacement plan.

This would turn exception handling from ad hoc branching into residual planning.

### 6. Async and actor adapters

Small adapters can make the typed batch boundary natural for:

- Tokio task groups;
- actor mailboxes;
- command buses;
- transactional outboxes;
- durable queue publishers;
- ECS command buffers.

These should remain optional crates or features. The core planning library should not acquire a mandatory async runtime.

### 7. Policy and authorization composition

A plan may be semantically valid but institutionally forbidden. The application boundary should support policy checks over typed commands before actuation, with refusal receipts that distinguish:

- planner capability refusal;
- application binding refusal;
- authorization refusal;
- stale-observation refusal;
- actuation failure.

### 8. Optimization and preferences

The current exact rail is a bounded satisfiability planner, not a general optimizing planner. Cost, preferences, metrics, temporal duration, and resource optimization need explicit semantic rails rather than being smuggled into action ordering.

### 9. Rich exact-state summaries

The witnessed concurrent rail exposes its final propositional state. The exact richer-classical rail currently exposes deterministic semantic replay and a semantic root rather than its internal rich numeric state. A stable, portable exact-state summary would improve downstream inspection while requiring careful canonicalization.

### 10. WASM, `no_std`, and constrained deployment profiles

The internal model is particularly valuable for edge and local software. Future profiles should make memory, action count, search depth, receipt size, and supported semantics explicit for WASM and constrained devices.

---

## Adoption ladder

The paradigm can be adopted incrementally.

### Stage 1: internal decision support

Plan and inspect typed commands, but let existing code choose whether to use them.

### Stage 2: command manufacture

Use the planner to create command batches while the existing service remains the durable executor.

### Stage 3: residual replanning

After each observed effect, update state and replan the remaining goal.

### Stage 4: receipt-linked actuation

Bind command and effect receipts to the planning execution root.

### Stage 5: planning-native application architecture

Represent major operational capabilities as action theories. Services, actors, devices, or aggregates manufacture their control flow from admitted observations and goals rather than accumulating hand-authored case trees.

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
