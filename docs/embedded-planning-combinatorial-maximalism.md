# Design for Combinatorial Maximalism: Embedded Planning as a Rust Control-Flow Compiler

## Status

This document is an architectural specification and implementation map. It distinguishes:

- **implemented foundation** — the current `EmbeddedWorkflow`, `WorkflowProblem`, deterministic STRIPS problem construction, verified PDDL → POWL execution, typed command binding, and receipt roots;
- **core-team surface** — reusable abstractions that should be owned by the project because they preserve semantic standing and multiply the number of lawful application compositions;
- **ecosystem surface** — runtime, framework, database, queue, actor, cloud, and business-specific adapters that must remain optional and downstream;
- **future semantic rails** — optimization, temporal, resource, preference, probabilistic, and continuous semantics that require explicit admission rather than convenience-layer approximation.

The objective is not to construct a larger workflow engine. The objective is to manufacture a **control-flow compiler platform** that Rust applications can embed, compose, inspect, verify, and actuate through their own authority boundary.

---

## 1. The CMD proposition

Design for Combinatorial Maximalism means maximizing the number of lawful systems that can be constructed from a small set of orthogonal, reversible, standing-preserving primitives.

For embedded planning, the useful composition space is:

```text
observation
    × action theory
    × goal
    × semantic rail
    × search policy
    × plan transformation
    × process geometry
    × command binding
    × institutional policy
    × execution adapter
    × cursor strategy
    × receipt backend
```

A monolithic workflow engine chooses most of these axes for the application. A combinatorial planning kernel exposes them as explicit values and contracts.

The design target is therefore not:

```text
one engine with many configuration flags
```

It is:

```text
small lawful primitives
    + typed composition
    + bounded transformations
    + replayable evidence
    = many application architectures
```

An illustrative surface with only four choices on each of eight independent axes already yields 65,536 possible lawful configurations. The point is not the number. The point is that the implementation cost grows roughly with the number of primitives, while application capability grows multiplicatively with their combinations.

### CMD law for this project

> Every new abstraction should either create a new independent composition axis, make an existing axis safer to combine, or remove accidental coupling between axes.

An abstraction should be refused when it:

- hides a semantic downgrade;
- combines decision manufacture with actuation;
- creates hidden engine state;
- introduces a mandatory runtime where a trait would suffice;
- converts a content-addressed artifact into an opaque handle without preserving identity;
- makes cross-process serialization look equivalent to replay verification;
- replaces typed refusals with generic retry behavior;
- forces one framework, database, queue, or async executor into the kernel.

---

## 2. The phase change: workflow engine → control-flow compiler

The correct architectural analogy is no longer a BPMN server or task orchestrator. It is a compiler toolchain.

```text
APPLICATION FRONT END
    domain state + action theory + goal
                │
                ▼
SEMANTIC FRONT END
    parse → canonicalize → admit → bind bounds
                │
                ▼
PLANNING MIDDLE END
    ground → search → validate → causalize → reduce → parallelize
                │
                ▼
PROCESS IR
    sequence + partial order + choice + bounded repetition
                │
                ▼
VERIFICATION
    execute → replay → receipt → standing
                │
                ▼
APPLICATION BACK END
    planner actions → native Rust commands
                │
                ▼
APPLICATION RUNTIME
    authorize → transact → dispatch → observe effects → receipt → replan
```

This compiler framing clarifies responsibility:

- PDDL is one front-end language for action theories and problems.
- POWL is the process intermediate representation.
- Semantic rails are middle-end strategies with different admitted meaning.
- Typed command binding is a backend.
- Tokio, actors, outboxes, queues, ECS, devices, and human approval are execution targets.
- Receipts are proof-carrying build artifacts for decisions and observed effects.
- Residual replanning is incremental recompilation against a changed world.

The application does not become a worker owned by an external engine. The application embeds a compiler for lawful future control flow.

That is the phase change:

> Control flow becomes a manufactured, inspectable, typed, receipt-bearing application artifact rather than either handwritten branching or externally owned workflow state.

---

## 3. Non-negotiable design laws

### 3.1 Zero unreceipted actuation

The kernel may parse, admit, plan, transform, execute process geometry, verify, and bind commands. It must not perform application side effects.

```text
planner output → proposal
verified POWL output → standing-bearing proposal
native command binding → typed proposal
broker admission → authority to attempt an effect
observed effect receipt → evidence of actuation
```

A verified plan is not a completed business operation.

### 3.2 Values before effects

Every boundary should produce a value that can be inspected before the next boundary is crossed:

- `AdmittedObservation`;
- `CompiledDomain`;
- `PlanRequest`;
- `CandidatePlan`;
- `VerifiedPlan`;
- `TypedPlan<C>`;
- `AuthorizedPlan<C>`;
- `DispatchProposal<C>`;
- `EffectObservation`;
- `ReplanDecision`.

### 3.3 Private manufacture of standing

Standing-bearing constructors remain private or sealed. Generic deserialization never recreates standing.

A type may be serializable for inspection or transport without being deserializable as trusted evidence.

### 3.4 Content-addressed identity

Stable semantic artifacts should identify their exact contents:

- domain source and compiled domain;
- observation snapshot;
- goal expression;
- planning bounds and policies;
- selected semantic rail;
- candidate plan;
- POWL process;
- command-binding schema;
- institutional policy set;
- dispatch proposal;
- observed effect;
- cursor state;
- receipt chain.

### 3.5 No semantic flattening

Convenience APIs may cover narrower surfaces, but may never encode richer semantics as weaker syntax.

Examples:

- a positive STRIPS builder does not represent negation as a specially named predicate;
- temporal duration is not smuggled into action labels;
- resources are not represented as undocumented ordering edges;
- preferences are not silently converted into goals;
- authorization is not represented as a planning precondition unless it truly belongs to the action theory.

### 3.6 Bounded by construction

Search, graph size, recursion, choices, loop count, receipt size, state projection, and adapter buffering must have explicit bounds.

Bounds are part of application policy and receipt identity, not incidental implementation constants.

### 3.7 Open adapters, closed kernel

The kernel defines traits and data contracts. Optional crates implement popular adapters. Application-specific handlers remain downstream.

### 3.8 Deterministic canonicalization

Equivalent admitted inputs should manufacture identical canonical forms and roots where semantics permit it.

### 3.9 Replay before trust

Crossing a process, storage, or organizational boundary converts standing-bearing data into an untrusted proposal unless a replay envelope, signature policy, or equivalent verification re-establishes standing.

### 3.10 No mandatory runtime

The core must not require Tokio, an actor framework, a database, a queue, a web server, or a cloud SDK.

---

## 4. Repository and crate topology

The repository already has the beginnings of the correct compiler split:

- `bcinr-mfw-ir` — domain-neutral semantic IDs, bounded outcomes, causal/concurrency contracts, and projection witnesses;
- `bcinr-pddl` — PDDL front end, grounding/search rails, cognitive routing, and embedded application facade;
- `bcinr-powl` — process IR, compiler, scheduler, and typestate;
- `bcinr-powl-receipt` — execution receipt and replay.

The next core-team topology should preserve that direction rather than creating a circular “workflow engine” crate.

### 4.1 Extend `bcinr-mfw-ir`

Own only language-independent contracts:

- semantic identity newtypes;
- standing and typestate markers;
- generic bounded collections;
- generic planner/pass outcomes;
- action occurrence and process references;
- receipt-link contracts;
- cursor and supersession identities;
- policy decisions that do not depend on a specific policy engine;
- generic observation, goal, and plan envelope metadata.

It must remain free of PDDL, POWL, Tokio, database, and queue dependencies.

### 4.2 Keep `bcinr-pddl` as a front end and semantic rail provider

Own:

- PDDL parsing and canonical source identity;
- action-theory/problem admission;
- STRIPS and exact-classical rails;
- PDDL-specific builders and macros;
- PDDL-specific problem projection utilities;
- conversion from admitted PDDL actions to domain-neutral action invocations;
- semantic routing and typed PDDL refusals.

### 4.3 Evolve `bcinr-powl` into the process middle end

Own:

- recursive process constructors;
- process normalization;
- partial-order validation and reduction;
- choice and bounded repetition;
- scheduling and guarded execution;
- process slicing, diffing, and residual process operations;
- process-level explanation and visualization;
- backend-neutral executable batches.

### 4.4 Add an application facade crate only when the compile graph warrants it

A future `bcinr-workflow` crate may become the stable public facade when the embedded surface spans multiple front ends. It should depend on IR and selected front ends, not the reverse.

It would own:

- `WorkflowKernel`;
- planning sessions;
- typed bindings;
- plan envelopes;
- cursors and resumability;
- policy composition contracts;
- residual replanning;
- broker-facing proposals.

Until that split is justified, these can remain modules behind the existing `bcinr-pddl` facade.

### 4.5 Optional adapter crates

Official adapters may include:

- `bcinr-workflow-tokio`;
- `bcinr-workflow-tower`;
- `bcinr-workflow-outbox`;
- `bcinr-workflow-actor`;
- `bcinr-workflow-ecs`;
- `bcinr-workflow-wasm`;
- `bcinr-workflow-testkit`.

None may become a dependency of the kernel.

---

## 5. Core abstraction catalog

The following is the maximal core-team surface. It is deliberately decomposed into independent axes.

## 5.1 Semantic identity

The core should provide strong newtypes instead of exchanging raw strings and digests.

```rust
pub struct DomainSourceRoot(Digest);
pub struct CompiledDomainRoot(Digest);
pub struct ObservationRoot(Digest);
pub struct GoalRoot(Digest);
pub struct BoundsRoot(Digest);
pub struct SearchPolicyRoot(Digest);
pub struct PlanRoot(Digest);
pub struct ProcessRoot(Digest);
pub struct ExecutionRoot(Digest);
pub struct BindingSchemaRoot(Digest);
pub struct PolicySetRoot(Digest);
pub struct DispatchRoot(Digest);
pub struct EffectRoot(Digest);
pub struct CursorRoot(Digest);
pub struct ReceiptRoot(Digest);
```

Utilities:

- `ContentAddressed` trait;
- canonical digest writer;
- domain-separated hashing;
- typed root parsing and formatting;
- root-chain builder;
- root comparison and mismatch diagnostics;
- `Versioned<T>` and `CompatibilityRange`;
- `ArtifactRef<T>` for references that do not imply possession or verification.

A raw `String` root should be confined to serialization boundaries.

## 5.2 Standing and typestate

The core should make illegal authority transitions difficult to express.

```rust
pub struct Candidate;
pub struct Parsed;
pub struct Admitted;
pub struct Planned;
pub struct ProcessVerified;
pub struct BoundToCommands;
pub struct PolicyAdmitted;
pub struct DispatchAdmitted;
pub struct EffectObserved;

pub struct Artifact<T, S> {
    value: T,
    root: Digest,
    standing: S,
}
```

Useful utilities:

- sealed transition traits;
- `map_value` that preserves standing only for identity-preserving transforms;
- `try_transform` that emits a new root and transition receipt;
- `erase_for_transport` that intentionally removes standing;
- `replay_and_restore` that re-establishes standing from complete evidence;
- compile-time distinction between proposal, verified decision, authorized work, and observed effect.

The goal is not typestate ornamentation. The goal is to encode the authority ladder.

## 5.3 Domain and action-theory abstractions

```rust
pub trait DomainSource {
    fn source_bytes(&self) -> Cow<'_, [u8]>;
    fn media_type(&self) -> &'static str;
}

pub trait DomainCompiler<S: DomainSource> {
    type Compiled;
    type Error;

    fn compile(&self, source: &S) -> Result<CompiledDomain<Self::Compiled>, Self::Error>;
}

pub struct CompiledDomain<D> {
    source_root: DomainSourceRoot,
    compiled_root: CompiledDomainRoot,
    version: DomainVersion,
    action_catalog: ActionCatalog,
    inner: D,
}
```

Core utilities:

- `DomainVersion`;
- `DomainCompatibility`;
- `DomainRegistry`;
- `DomainSet` for applications carrying several action theories;
- `DomainSelector<Context>`;
- `DomainMigration`;
- `PredicateCatalog`;
- `FluentCatalog`;
- `ActionCatalog`;
- `ResourceCatalog` when resource semantics are admitted;
- action signature lookup;
- deterministic domain rendering;
- domain diff and compatibility report;
- persisted-plan compatibility check.

The registry stores semantic artifacts, not mutable execution cursors.

## 5.4 Observation abstractions

`WorkflowProblem` is the correct first boundary. CMD expands it into snapshots, deltas, views, and admission.

```rust
pub trait ObservationProjection<State> {
    type Observation;
    type Error;

    fn project(&self, state: &State) -> Result<Self::Observation, Self::Error>;
}

pub struct ObservationSnapshot<O> {
    observed_at: LogicalTime,
    source_version: SourceVersion,
    root: ObservationRoot,
    value: O,
}

pub struct ObservationDelta<D> {
    base: ObservationRoot,
    root: ObservationRoot,
    delta: D,
}
```

Core utilities:

- `ObservationView` for bounded borrowed projections;
- `AdmittedObservation` with explicit semantic profile;
- `ObservationDelta` and `ObservationPatch`;
- snapshot/delta canonicalization;
- root-aware merge and conflict detection;
- stale-observation predicates;
- observation redaction before persistence;
- deterministic logical time;
- source-version binding;
- observation-size and fact-count bounds;
- exact observation diff;
- relevant-fact slicing;
- dependency mapping from facts to planned actions.

Observation projection must remain application-owned. The kernel may validate the projection; it must not introspect arbitrary application state.

## 5.5 Goal algebra

Goals should be values rather than unstructured strings.

```rust
pub enum GoalExpr<A, N> {
    Atom(A),
    Not(Box<Self>),
    All(Vec<Self>),
    Any(Vec<Self>),
    Exists { variable: String, body: Box<Self> },
    ForAll { variable: String, body: Box<Self> },
    Numeric(N),
}

pub struct GoalEnvelope<G> {
    goal: G,
    priority: GoalPriority,
    deadline: Option<LogicalDeadline>,
    policy: GoalPolicy,
    root: GoalRoot,
}
```

Core utilities:

- `GoalSet`;
- `GoalPriority`;
- hard versus soft goal distinction;
- goal normalization;
- goal implication checks where admitted;
- goal diff;
- residual goal calculation;
- goal satisfaction explanation;
- goal template/factory;
- goal source identity;
- goal-bound validation;
- explicit refusal when a front end cannot represent a goal expression.

The generic algebra does not imply that every semantic rail supports every constructor.

## 5.6 Planning request and session

```rust
pub struct PlanRequest<D, O, G> {
    domain: ArtifactRef<D>,
    observation: O,
    goal: G,
    bounds: PlanningBounds,
    policy: PlanningPolicy,
    request_root: Digest,
}

pub struct PlanningSession<K> {
    kernel: K,
    session_id: PlanningSessionId,
    cache_scope: CacheScope,
    logical_clock: LogicalClock,
}
```

Core utilities:

- `PlanRequestBuilder`;
- request canonicalization;
- request root;
- deadline and cancellation token abstraction without requiring an async runtime;
- `PlanningBounds` and typed sub-bounds;
- `PlanningPolicy`;
- `PlanMode` such as first-valid, all-bounded, optimize-under-profile, or explain-only;
- tenant/cache scope;
- deterministic seed identity where a rail uses controlled randomness;
- request audit summary;
- request equivalence check;
- request-size accounting.

`EmbeddedWorkflow` can evolve toward or wrap this model while retaining the simple path.

## 5.7 Semantic rail contracts

```rust
pub trait SemanticRail<Request> {
    type Candidate;
    type Standing;
    type Refusal;

    fn admit_and_plan(
        &mut self,
        request: &Request,
    ) -> PlannerOutcome<Artifact<Self::Candidate, Self::Standing>>;
}
```

Core utilities:

- `RailId` and `RailVersion`;
- `RailCapabilityProfile`;
- `RailRouter`;
- `RailPortfolio`;
- `RailSelectionReceipt`;
- `SearchBudget`;
- `SearchCheckpoint`;
- `SearchTrace`;
- deterministic fairness scheduler;
- typed fallback rules;
- explicit “fallback only on unsupported capability” combinator;
- rail comparison harness;
- result equivalence checker;
- bound-exhaustion witnesses;
- search-state accounting;
- rail-level metrics that do not change semantics.

The router must not convert parse, consistency, validation, receipt, or projection failure into fallback success.

## 5.8 Plan-pass algebra

Compiler-style passes are the heart of combinatorial maximalism.

```rust
pub trait PlanPass<I> {
    type Output;
    type Witness;
    type Refusal;

    fn apply(&self, input: I)
        -> Result<PassOutput<Self::Output, Self::Witness>, Self::Refusal>;
}

pub struct PassOutput<T, W> {
    value: T,
    witness: W,
    input_root: Digest,
    output_root: Digest,
}
```

Core passes and utilities:

- parse;
- canonicalize;
- admit;
- ground;
- search;
- validate selected plan;
- replay state transitions;
- causal support extraction;
- conflict analysis;
- transitive reduction;
- independence proof;
- partial-order construction;
- maximal-batch extraction;
- process projection;
- process compilation;
- guarded scheduling;
- receipt verification;
- dead-action elimination;
- common-prefix factoring;
- common-suffix factoring;
- plan slicing;
- plan splicing;
- plan diff;
- plan patch;
- residualization;
- exact suffix validation;
- explanation generation;
- visualization projection.

Pass composition should be explicit:

```rust
let pipeline = parse
    .then(canonicalize)
    .then(admit)
    .then(search)
    .then(causalize)
    .then(project_powl)
    .then(verify);
```

The combinator should preserve typed input/output compatibility and accumulate a receipt chain.

## 5.9 Process algebra and POWL utilities

The recursive POWL model supplies the correct base constructors:

- activity;
- silent activity;
- sequence;
- partial order;
- generalized choice graph;
- bounded do-redo.

CMD requires a complete manipulation toolkit around those constructors.

```rust
pub struct Process<A> {
    root: ProcessRoot,
    model: PowlModel<A>,
}

pub struct ProcessSlice<A> {
    parent: ProcessRoot,
    selected_nodes: NodeSet,
    process: Process<A>,
}
```

Core utilities:

- ergonomic constructor functions and builders;
- typed node references instead of raw indexes;
- stable process node IDs;
- recursive validation;
- normalization;
- alpha-renaming of node identities;
- sequence flattening;
- silent-node elimination where semantics permit;
- partial-order cycle diagnostics;
- transitive closure and reduction;
- topological-layer extraction;
- antichain enumeration under bounds;
- maximal ready-set calculation;
- critical path;
- concurrency width;
- dominator/post-dominator analysis for choice graphs;
- reachability and start/end coverage;
- strongly connected component analysis for bounded loops;
- process slicing by action, fact, goal, tick, or receipt;
- process diff;
- process merge with conflict witnesses;
- common-subprocess factoring;
- replacement/splice operations with validation;
- choice-policy identity and selection receipts;
- process metrics;
- DOT, Mermaid, JSON, and compact binary projections;
- stable human explanation.

Future constructors such as resource barriers, temporal windows, cancellation scopes, races, or compensation regions must be added only with explicit semantics and witnesses.

## 5.10 Action invocation and binding

`ActionInvocation` and `TryFrom<ActionInvocation>` are the correct minimal bridge. The core team should generalize this into a binding contract with schema identity.

```rust
pub trait ActionBinding {
    type Command;
    type Error;

    fn schema_root(&self) -> BindingSchemaRoot;
    fn bind(&self, action: &ActionInvocation)
        -> Result<Self::Command, Self::Error>;
}

pub struct BindingRegistry<B> {
    bindings: B,
    schema_root: BindingSchemaRoot,
}
```

Core utilities:

- typed action names and arguments;
- positional and named argument views;
- argument-count validation;
- typed identifier parsers;
- binding registry;
- exhaustive action-catalog coverage check;
- duplicate-binding refusal;
- command schema identity;
- binding compatibility report;
- command redaction policy;
- command display and debug summaries;
- one-way serialization of standing-bearing typed plans;
- explicit untrusted transport DTO;
- re-binding verification after transport;
- `bind`, `map_actions`, and registry-based binding;
- generated bindings from an admitted domain model.

Procedural macros may provide:

```rust
#[derive(PlanningCommand)]
#[planning_domain(DOMAIN)]
enum FulfillmentCommand {
    #[planning_action("reserve-inventory")]
    ReserveInventory { order: OrderId },
}
```

The macro must generate validation and tests, not hide stringly dispatch behind generated code.

## 5.11 Policy composition

A semantically valid plan can still be institutionally forbidden. Policy must be a separate axis.

```rust
pub trait Policy<Input, Context> {
    type Evidence;
    type Refusal;

    fn evaluate(
        &self,
        input: &Input,
        context: &Context,
    ) -> PolicyDecision<Self::Evidence, Self::Refusal>;
}
```

Core combinators:

- `All<P>`;
- `Any<P>`;
- `Not<P>`;
- `AndThen<P, Q>`;
- `MapEvidence<P, F>`;
- `ContramapInput<P, F>`;
- `Quorum<P>`;
- `NamedPolicy<P>`;
- `VersionedPolicy<P>`;
- `PolicySet`;
- `PolicyDecision`;
- refusal accumulation;
- policy-root manufacture;
- deterministic evaluation order;
- evidence redaction;
- policy diff;
- policy compatibility check.

Policy evaluation does not actuate. It manufactures an authorization proposal or refusal receipt.

## 5.12 Broker-facing command envelopes

The kernel should standardize the boundary without owning the implementation.

```rust
pub struct CommandEnvelope<C> {
    plan_root: PlanRoot,
    execution_root: ExecutionRoot,
    binding_root: BindingSchemaRoot,
    policy_root: Option<PolicySetRoot>,
    tick: u32,
    command_index: u32,
    command: C,
}

pub struct DispatchProposal<C> {
    root: DispatchRoot,
    commands: Vec<CommandEnvelope<C>>,
    idempotency: IdempotencyKey,
}
```

Core traits:

```rust
pub trait BatchBroker<C> {
    type Admission;
    type Refusal;

    fn admit_batch(
        &mut self,
        proposal: &DispatchProposal<C>,
    ) -> Result<Self::Admission, Self::Refusal>;
}

pub trait EffectObserver<C> {
    type Observation;
    type Error;

    fn observe_effect(
        &mut self,
        command: &CommandEnvelope<C>,
    ) -> Result<Self::Observation, Self::Error>;
}
```

Companion contracts:

- `IdempotencyStore`;
- `TransactionBoundary`;
- `OutboxSink`;
- `CommandSink`;
- `ReceiptSink`;
- `EffectSource`;
- `Clock`;
- `RetryPolicy`;
- `CompensationPolicy`;
- `DeadLetterSink`;
- `ApprovalGate`.

The core supplies interfaces, envelopes, roots, and test doubles. It does not supply business command handlers.

## 5.13 Cursor and resumability

The cursor must be explicit application data rather than hidden engine state.

```rust
pub struct WorkflowCursor {
    plan_root: PlanRoot,
    process_root: ProcessRoot,
    next_tick: u32,
    command_states: Vec<CommandProgress>,
    generation: u32,
    superseded_by: Option<PlanRoot>,
    root: CursorRoot,
}

pub enum CommandProgress {
    Pending,
    Admitted { dispatch_root: DispatchRoot },
    Attempted { attempt: u32 },
    EffectObserved { effect_root: EffectRoot },
    Refused { refusal_root: ReceiptRoot },
    Compensated { effect_root: EffectRoot },
}
```

Core utilities:

- `CursorBuilder`;
- next-ready-batch calculation;
- partial-batch policy;
- all-or-nothing batch policy;
- retry generation;
- cursor root and transition receipt;
- cursor validation against plan root;
- cursor serialization as untrusted data;
- replay verification;
- supersession;
- resume token;
- cursor compaction;
- cursor diff;
- completed-effect extraction;
- pending-work extraction;
- abandonment and cancellation receipts.

## 5.14 Residual replanning

Residual replanning is the runtime counterpart of incremental compilation.

```rust
pub struct ResidualRequest<O, G> {
    original_plan: PlanRoot,
    original_observation: ObservationRoot,
    current_observation: O,
    completed_effects: Vec<EffectRoot>,
    residual_goal: G,
}

pub enum ReplanDecision<P> {
    KeepSuffix { suffix: P, witness: SuffixValidityWitness },
    Replace { plan: P, diff: PlanDiff },
    GoalAlreadySatisfied { witness: GoalWitness },
    Refuse { reason: ReplanRefusal },
}
```

Core utilities:

- stale-plan detection;
- completed-prefix validation;
- still-valid suffix validation;
- residual-goal manufacture;
- residual-observation projection;
- plan suffix extraction;
- replacement plan manufacture;
- old/new plan diff;
- command cancellation set;
- reusable-command set;
- supersession receipt;
- replan generation numbering;
- convergence and churn bounds;
- policy-controlled replan triggers;
- exact-match consequence cache integration.

This turns exception handling from a growing tree of handwritten cases into bounded recompilation against the new world.

## 5.15 Receipt algebra

The receipt system should form a typed chain rather than a collection of unrelated JSON logs.

```rust
pub enum WorkflowReceipt {
    Domain(DomainReceipt),
    Observation(ObservationReceipt),
    Goal(GoalReceipt),
    Admission(AdmissionReceipt),
    Search(SearchReceipt),
    Plan(PlanReceipt),
    Process(ProcessReceipt),
    Execution(ExecutionReceipt),
    Binding(BindingReceipt),
    Policy(PolicyReceipt),
    Dispatch(DispatchReceipt),
    Effect(EffectReceipt),
    Cursor(CursorReceipt),
    Replan(ReplanReceipt),
}
```

Core utilities:

- parent-root chaining;
- receipt envelope versioning;
- canonical receipt encoding;
- receipt redaction;
- detached signatures through an optional trait;
- receipt set validation;
- replay plan;
- missing-parent diagnostics;
- tamper localization;
- compact receipt summaries;
- receipt indexing;
- causal query: “which observation and goal caused this command?”;
- reverse query: “which effects satisfy this goal?”;
- receipt retention policy;
- export to OCEL and public provenance vocabularies where appropriate.

The core should define signature interfaces without imposing a specific key store or PKI.

## 5.16 Refusal taxonomy

Every boundary needs a refusal type precise enough for policy and recovery.

Top-level categories:

- source parse refusal;
- canonicalization refusal;
- unsupported semantic capability;
- inconsistent theory/problem;
- invalid observation projection;
- invalid goal;
- bound exhaustion;
- search exhaustion;
- selected-plan validation failure;
- causal-analysis failure;
- concurrency-witness failure;
- process-projection failure;
- process validation failure;
- scheduler deadlock;
- execution-replay mismatch;
- action-label refusal;
- command-binding refusal;
- policy refusal;
- stale-observation refusal;
- broker admission refusal;
- idempotency conflict;
- actuation failure;
- effect-observation failure;
- cursor mismatch;
- replan refusal;
- receipt mismatch;
- transport trust refusal.

Core utilities:

- stable refusal codes;
- source chains;
- machine-readable context;
- human explanation;
- retry classification;
- redaction;
- conversion only when semantics are preserved;
- refusal receipt manufacture.

A generic `WorkflowError` may wrap categories for ergonomics, but must not erase them.

---

## 6. Foundational utility modules

These utilities create disproportionate combinatorial value because every semantic rail and adapter can reuse them.

## 6.1 Bounded collections

- `BoundedVec<T, const N: usize>`;
- `BoundedSet<T, const N: usize>`;
- `BoundedMap<K, V, const N: usize>`;
- `NodeSet` bitsets;
- `ActionSet` bitsets;
- capacity-refusal witnesses;
- deterministic iteration;
- compact serialization.

## 6.2 Canonical data utilities

- stable sort and dedup;
- canonical map/set rendering;
- canonical S-expression writer;
- canonical JSON profile where required;
- deterministic binary encoding;
- domain-separated hashing;
- normalized symbol interner;
- case and namespace policy;
- canonical float/fixed-point policy;
- canonical logical time.

## 6.3 Graph utilities

- cycle detection with witness;
- topological sort;
- transitive closure;
- transitive reduction;
- reachability;
- SCC decomposition;
- antichains;
- maximal ready sets;
- dominators;
- critical path;
- graph slicing;
- graph diff;
- graph patch;
- graph merge with conflict witness;
- bounded path enumeration;
- minimal cut/support sets;
- conflict graph construction;
- maximal independent sets under explicit bounds.

## 6.4 Plan utilities

- selected-action indexing;
- causal-support lookup;
- action-to-goal contribution map;
- action-to-observation dependency map;
- unused-action detection;
- prefix/suffix extraction;
- state replay;
- exact post-state summary;
- plan equivalence under admitted semantics;
- plan diff;
- process diff;
- explanation views;
- stable pretty printing;
- compact summaries.

## 6.5 Deterministic environment utilities

- logical clock;
- deterministic seed source;
- bounded cancellation token;
- resource meter;
- allocation meter;
- deadline abstraction;
- trace collector;
- no-op and recording implementations for tests.

---

## 7. Developer-facing ergonomics

CMD does not mean exposing every internal type to every application. It means providing progressive disclosure over the same lawful kernel.

### 7.1 Level 1: simple embedded planning

```rust
let mut workflow = EmbeddedWorkflow::new(DOMAIN)?;
let commands = workflow
    .plan(&state)?
    .bind::<Command>()?;
```

### 7.2 Level 2: explicit request and bounds

```rust
let request = workflow
    .request(&state)
    .goal(OrderGoal::Fulfilled)
    .bounds(PlanningBounds::interactive())
    .build()?;

let plan = workflow.plan_request(request)?;
```

### 7.3 Level 3: policy and broker proposal

```rust
let typed = plan.bind_with(&FulfillmentBinding)?;
let authorized = policy_set.evaluate(&typed, &actor_context)?;
let proposal = DispatchProposal::from_authorized(authorized)?;
let admission = broker.admit_batch(&proposal)?;
```

### 7.4 Level 4: durable cursor and residual replanning

```rust
let mut cursor = WorkflowCursor::new(&typed)?;
let batch = cursor.next_ready_batch()?;
let admission = broker.admit_batch(&batch)?;
cursor.record_admission(admission)?;

let effects = observer.collect(&cursor)?;
let decision = replanner.reconcile(&cursor, &new_state, effects)?;
```

### 7.5 Level 5: custom compiler pipeline

```rust
let pipeline = Pipeline::new()
    .pass(ParsePddl)
    .pass(Canonicalize)
    .pass(AdmitWith(profile))
    .rail_portfolio(rails)
    .pass(Causalize)
    .pass(ProjectPowl)
    .pass(VerifyExecution)
    .backend(binding);
```

All five levels must manufacture compatible roots and evidence. Ergonomics may hide plumbing, not meaning.

---

## 8. Macros and code generation

A core team would eventually write macros because they remove accidental duplication across the domain, planner labels, Rust commands, tests, and receipts.

Candidate surfaces:

- `planning_domain!` — construct a domain from a Rust DSL while preserving an emitted canonical source artifact;
- `goal!` — typed goal construction;
- `facts!` — bounded fact construction;
- `#[derive(WorkflowProblem)]` — projection from annotated fields under an explicit projection profile;
- `#[derive(PlanningCommand)]` — action binding and schema identity;
- `#[planning_action]` — action name and argument mapping;
- `#[planning_test]` — scenario harness generation;
- `include_planning_domain!` — compile-time source inclusion plus digest constant;
- `assert_domain_bindings_complete!` — test-time coverage proof;
- `assert_plan!` — semantic plan assertions without brittle string snapshots.

Macro laws:

- generated artifacts must be inspectable;
- generated source and roots must be reproducible;
- the macro must not become a second shadow semantic model;
- compile errors should identify domain/action/argument mismatches;
- richer semantics require explicit DSL constructors;
- no handler invocation is generated inside planning.

---

## 9. Testkit and verification utilities

A planning-native application needs stronger tests than “activity X was queued.”

The core testkit should provide:

### Scenario construction

- domain fixture;
- observation fixture;
- goal fixture;
- bound profile;
- expected standing;
- expected refusal;
- effect script;
- replan script.

### Assertions

- `assert_standing!`;
- `assert_action_present!`;
- `assert_action_absent!`;
- `assert_before!`;
- `assert_parallel!`;
- `assert_conflict!`;
- `assert_goal_satisfied!`;
- `assert_binding_complete!`;
- `assert_root_stable!`;
- `assert_replay_valid!`;
- `assert_replan_reuses!`;
- `assert_tamper_refused!`;
- `assert_bound_hit!`.

### Deterministic fakes

- recording broker;
- refusing broker;
- idempotency store;
- logical clock;
- effect observer;
- receipt sink;
- policy evaluator;
- cursor store;
- deterministic choice policy.

### Property strategies

- small valid domains;
- small valid problems;
- independent action sets;
- conflict pairs;
- equivalent insertion orders;
- process graphs;
- choice graphs;
- bounded loops;
- receipt mutations;
- observation deltas;
- replan scenarios.

### Metamorphic properties

- canonical insertion order does not change roots;
- transitive reduction preserves admitted executions;
- independent action reordering preserves final state;
- serialization does not recreate standing;
- replay restores standing only with complete evidence;
- a changed domain root invalidates persisted-plan compatibility;
- a completed valid prefix is preserved by residualization when possible;
- an unsupported rich feature never reaches the STRIPS compatibility rail as success.

---

## 10. Optional adapter matrix

The adapter interface is another combinatorial axis. Each adapter consumes the same typed batches and roots.

| Adapter | Core projection | Adapter-owned concerns |
|---|---|---|
| Tokio | batch → task group | spawning, cancellation, join policy |
| Tower | command → service request | readiness, backpressure, middleware |
| Actor | command → mailbox message | supervision, mailbox ordering, actor identity |
| Transactional outbox | batch → outbox records | SQL transaction, schema, polling |
| Durable queue | command → publication | delivery semantics, acknowledgements, DLQ |
| ECS | batch → command buffer | world borrowing, stage boundaries |
| Device | command → device operation proposal | hardware safety, drivers, watchdogs |
| Human approval | batch → approval packet | UI, identity, signatures, timeout |
| WASM host | command → host-call proposal | capability grants, ABI, sandbox limits |
| External workflow service | batch → activity submission | remote durability and provider semantics |

The final row is important. External workflow systems do not disappear. They become optional execution backends rather than owners of reasoning and process meaning.

---

## 11. Core-team boundary

### The core team should own

- identity and canonicalization;
- semantic standing and typestate;
- domain/front-end contracts;
- observation and goal envelopes;
- bounded planning requests;
- rail and pass contracts;
- POWL process algebra and graph utilities;
- verified execution and replay;
- action binding contracts and schema identity;
- policy composition contracts;
- dispatch proposal envelopes;
- cursor and residual replanning semantics;
- receipt algebra;
- refusal taxonomy;
- deterministic testkit;
- optional reference adapters.

### The core team should not own

- application command handlers;
- database-specific business repositories;
- tenant authorization rules;
- cloud credentials;
- queue topics and routing policy;
- device drivers;
- business compensation logic;
- UI approval flows;
- arbitrary application-state introspection;
- hidden retry loops;
- a mandatory daemon or hosted control plane.

This boundary preserves the phase change. The kernel manufactures lawful control flow; the application retains sovereignty over reality.

---

## 12. The combinatorial payoff

Consider a modest library installation with:

- three observation projections;
- two domain versions;
- four goal templates;
- two semantic rails;
- three search policies;
- four plan-pass profiles;
- two binding schemas;
- three institutional policy sets;
- five broker adapters;
- two cursor policies.

Those small independent surfaces permit 34,560 application configurations before considering different action theories or concrete states.

A monolithic engine usually turns this space into configuration conditionals maintained by the engine team. CMD turns it into typed application composition maintained at the appropriate boundary.

More importantly, the same primitive can participate in contexts the core team did not anticipate:

- a game AI can use the observation and plan-pass algebra but dispatch through ECS;
- a financial service can use the same planning rail and process IR but dispatch through an outbox;
- an edge device can use the same domain and command binding offline;
- a human case-management system can replace automated dispatch with approval packets;
- a build system can treat repository state as observations and tool invocations as typed commands;
- an external workflow platform can remain the durable executor while no longer owning process manufacture.

That is combinatorial maximalism in operational form: capability comes from lawful recombination, not from putting every use case into the kernel.

---

## 13. Implementation waves

### Wave 0 — current foundation

Already present:

- resident embedded domain;
- application-state problem projection;
- deterministic STRIPS builder;
- verified semantic rails;
- POWL batches;
- execution roots;
- parsed action invocations;
- typed command binding;
- one-way serialization of standing-bearing values;
- developer quickstart and runnable example.

### Wave 1 — identity and binding hardening

Implement:

- typed semantic root newtypes;
- `ContentAddressed`;
- `ActionBinding` with schema root;
- action-catalog binding coverage check;
- `PlanRequest` and `PlanningBounds` value objects;
- explicit untrusted transport DTOs;
- exact domain/binding compatibility report.

Acceptance:

- no public API exchanges a planning/execution root as an untyped string;
- persisted typed work names its domain, rail, bounds, and binding roots;
- every action can be checked against a binding catalog before runtime planning.

### Wave 2 — pass algebra and process utilities

Implement:

- generic `PlanPass`;
- pass receipt chain;
- POWL constructor helpers;
- stable node references;
- process normalize/slice/diff;
- transitive-reduction utility;
- causal explanation queries;
- DOT/Mermaid projection.

Acceptance:

- passes compose without dynamic downcasting;
- each semantics-changing pass emits a witness and new root;
- process transformations replay to equivalent admitted behavior under their declared law.

### Wave 3 — explicit cursor and receipts

Implement:

- `CommandEnvelope`;
- `DispatchProposal`;
- `WorkflowCursor`;
- cursor transition receipts;
- receipt parent chaining;
- replay verification;
- recording broker and receipt sink.

Acceptance:

- a workflow can stop after any tick, persist untrusted cursor data, replay, and resume;
- completed effects cannot be confused with admitted or attempted commands;
- superseded plans remain linked rather than overwritten.

### Wave 4 — residual replanning

Implement:

- stale-observation policy;
- completed-prefix extraction;
- suffix validity checking;
- residual-goal manufacture;
- replacement plan and diff;
- supersession receipts;
- consequence-cache integration.

Acceptance:

- changed state either preserves a verified suffix, manufactures a replacement, reports goal completion, or emits a typed refusal;
- replanning has explicit churn and generation bounds.

### Wave 5 — macros and testkit

Implement:

- command-binding derive;
- domain inclusion macro with digest constant;
- binding-completeness assertion;
- scenario harness;
- broker/policy/effect fakes;
- graph and receipt property strategies;
- tamper tests.

Acceptance:

- an external crate can define a small domain, derive bindings, plan, inspect, and test without hand-written PDDL string assembly or stringly command dispatch;
- generated artifacts remain inspectable and reproducible.

### Wave 6 — optional runtime adapters

Implement adapters independently:

- Tokio task groups;
- Tower services;
- transactional outbox;
- actor mailbox;
- ECS command buffer;
- WASM host proposals.

Acceptance:

- no adapter is required by the core;
- adapters consume the same command envelope and produce the same receipt contracts;
- adapter failure cannot retroactively change planning standing.

### Wave 7 — richer semantic rails

Add only with explicit models and witnesses:

- optimization and cost;
- preferences;
- temporal duration;
- resources;
- deadlines;
- continuous change;
- probabilistic outcomes;
- contingent observation and policy branching.

Acceptance:

- each rail publishes its admitted feature profile;
- no richer rail is projected into a weaker standing without a documented conservative theorem or explicit refusal;
- process and receipt types represent the additional semantics directly.

---

## 14. Deterministic next tickets

1. **TypedRoots** — replace application-facing string roots with semantic newtypes and conversion utilities.
2. **BindingSchema** — add `ActionBinding`, binding root, and catalog coverage validation.
3. **PlanEnvelope** — manufacture a portable metadata envelope binding domain, observation, goal, bounds, rail, process, execution, and binding identities.
4. **UntrustedTransport** — define transport DTOs that cannot be mistaken for verified plans.
5. **PassChain** — add typed plan-pass composition with input/output roots and witnesses.
6. **ProcessNodeRef** — replace external raw POWL child indexes with stable validated node references.
7. **ProcessToolkit** — normalize, slice, diff, transitive-reduce, explain, and render POWL processes.
8. **CommandEnvelope** — bind typed commands to plan/execution roots, ticks, and command indices.
9. **BrokerContracts** — define broker, idempotency, receipt sink, and effect observer traits plus recording fakes.
10. **WorkflowCursor** — persist explicit tick/command progress and transition receipts.
11. **ReceiptChain** — link planning, binding, policy, dispatch, effect, cursor, and replan receipts.
12. **ResidualReplan** — preserve valid suffixes or manufacture replacement plans from changed observations.
13. **BindingDerive** — generate exhaustive command binding and schema identity from annotated Rust enums.
14. **WorkflowTestkit** — scenarios, semantic assertions, tamper tools, and property strategies.
15. **TokioAdapter** — map admitted batches to task groups without changing core semantics.
16. **OutboxAdapter** — map admitted batches to transactional outbox records with execution-root linkage.

Each ticket should end in public API tests from an external consumer crate and a focused verifier obligation.

---

## 15. Final paradigm statement

The first phase change was moving workflow and planning from an external control plane into the Rust application.

Design for Combinatorial Maximalism creates the second phase change:

> Embedded planning stops being one integrated library call and becomes a composable compiler architecture for manufacturing application control flow.

The application can independently choose:

- what it observes;
- which action theory it carries;
- which goal it asserts;
- which semantic rail it admits;
- which bounded transformations it applies;
- which process geometry it preserves;
- which native command vocabulary it targets;
- which institutional policy evaluates the commands;
- which broker may attempt effects;
- which cursor and replan policy govern change;
- which receipt system establishes standing.

Those choices combine without relocating authority to a workflow server and without allowing the planner to mutate the world.

The resulting primitive is larger than “workflow” and more general than “planning”:

```text
observe
    → admit
    → manufacture possible control flow
    → preserve process geometry
    → verify
    → bind to native commands
    → authorize
    → propose actuation
    → observe effects
    → receipt
    → residualize and replan
```

This is a planning-native Rust application architecture.

Code no longer has to enumerate every path. The core team supplies the lawful algebra, compiler passes, standing transitions, and evidence contracts. Application teams define their domain, state, goals, commands, policies, and authority over effects.

The combinatorial result is not one universal workflow engine. It is an ecosystem in which verified control-flow manufacture becomes as ordinary and reusable as parsing, query planning, serialization, or type checking.
