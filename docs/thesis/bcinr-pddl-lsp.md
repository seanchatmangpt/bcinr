# bcinr-pddl-lsp: The Lifecycle Map as Language Server

## Abstract

We present **bcinr-pddl-lsp**, a language server that makes project planning mechanical. Where bcinr-pddl provides the bounded, receipted execution engine, bcinr-pddl-lsp provides the projection layer: it reads a project workspace, extracts lifecycle facts from files on disk, generates a valid PDDL8 domain and problem, delegates planning and receipted execution to bcinr-pddl, and exposes the resulting plan, blockers, receipts, and OCEL events as LSP diagnostics, code actions, and virtual documents. The design inverts the normal relationship between language servers and editors: the language server does not merely describe a language — it shapes the project by admitting only lifecycle states and transitions that can be projected into bounded PDDL8 and executed under Prolog8 admission. The result is a mechanical answer to the question every agent and human faces: *what is the next lawful step toward publish?*

---

## 1. The Problem This Solves

Modern AI coding agents are capable planners in the semantic sense. Given a description of a task, they can propose a sequence of steps, write code, run tests, observe failures, and iterate. What they cannot do is *mechanically plan project execution under a policy* — they work from confidence, not from receipts.

This creates a structural gap:

```
Agent intention  ──→  agent action  ──→  project state
    (prose)            (unreceipted)       (opaque)
```

No one in this loop can answer the following questions mechanically:

- Is the PRD admitted?
- Does the ARD exist and does it derive from the admitted PRD?
- What is the next lawful lifecycle step given current project state?
- Which steps toward publish are blocked?
- What plan reaches `published(project)` from here?
- Was each step in that plan admitted by a Prolog8 policy?
- What receipt proves the plan executed?
- What OCEL trace records what happened?

bcinr-pddl-lsp answers all of these questions. It does so not by reasoning — by projecting.

---

## 2. Inverted LSP

The standard mental model of a language server is: human writes code, language server reads it, language server tells the human what is wrong. The server is reactive and passive. The authority is external: language specifications, type systems, APIs.

bcinr-pddl-lsp inverts this model on two axes.

**Axis 1: The primary client is not a human.** The primary consumers of bcinr-pddl-lsp diagnostics, code actions, and virtual documents are AI agents operating in Claude Code, MCP tool servers, and CI pipelines. These clients do not need editor integration to benefit from the LSP — they need the *protocol surface*. LSP is the right abstraction because it provides a standardized way to emit structured feedback (diagnostics), offer repair actions (code actions), query project state (virtual documents), and push state changes (notifications) to any client that speaks JSON-RPC.

**Axis 2: The server shapes the project.** A normal language server describes what exists. bcinr-pddl-lsp enforces what *must* exist before publish is admitted. A missing PRD is not a style warning — it is a `PRD_MISSING` ERROR that blocks the publish gate. An un-admitted ARD is not a note — it is a `ARD_NOT_ADMITTED` WARNING that blocks `derive_ard` from appearing in the plan. The server does not merely observe the project; it holds the lifecycle law.

This is the BRCE principle expressed in LSP terms:

```
actuate(publish) ⟺ R ⊢ published(project)
```

The LSP enforces that `R ⊢ published(project)` is only derivable through the full lifecycle plan, admitted by Prolog8, receipted by BLAKE3.

---

## 3. Architecture

The server is organized into eight modules with a strict dependency order:

```
lifecycle  ←  workspace files on disk
    ↓
bounds     ←  PDDL8/Need9 bound checks
    ↓
projection ←  lifecycle facts → PDDL8 domain + problem
    ↓
planner_client  ←  bcinr-pddl (parse, ground, BFS, execute, receipt, OCEL)
    ↓
publish_gate    ←  OPEN / PARTIAL / BLOCKED / ADMITTED / PUBLISHED
    ↓
diagnostics     ←  lifecycle/planning errors → LSP diagnostic codes
virtual_docs    ←  project map rendered as bcinr-pddl:// URIs
code_actions    ←  repair actions per diagnostic
    ↓
backend         ←  lsp-max LanguageServer impl (didOpen, didChange, execute_command)
```

No module calls upward. The backend orchestrates; the modules are pure transformations of data.

### 3.1 lifecycle

The lifecycle scanner is the first computation in every analysis cycle. It walks the workspace root and looks for evidence of each lifecycle stage:

| Stage | Evidence |
|---|---|
| `intent_captured` | `README.md`, `CLAUDE.md`, `intent.md` present |
| `prd_exists` | `docs/prd.md` or `PRD.md` present |
| `prd_admitted` | PRD file contains the string `ADMITTED` |
| `ard_exists` | `docs/ard.md` or `ARD.md` present |
| `ard_admitted` | ARD file contains the string `ADMITTED` |
| `work_units_generated` | `docs/work-units.md` or `.bcinr/work-units.json` present |
| `implementation_complete` | `.rs` files found under `src/` or `crates/` (depth ≤ 4) |
| `tests_passed` | `.bcinr/test-report.json` contains `"passed": true` |
| `docs_projected` | `docs/` contains ≥ 1 `.md` file beyond `prd.md`/`ard.md` |
| `release_ready` | `.bcinr/release.json` or `docs/publish.md` present |
| `published` | `.bcinr/receipts/latest.json` contains `"goal_reached": true` |

The scanner is intentionally file-centric: it reads evidence from the filesystem, not from agent claims. An agent that says "I ran the tests" without producing a test report does not advance `tests_passed`. A PRD that says "this is admitted" without the word `ADMITTED` does not advance `prd_admitted`. The lifecycle is grounded in artifacts, not assertions.

The output is `ProjectLifecycle`:

```rust
pub struct ProjectLifecycle {
    pub project_name: String,
    pub root: PathBuf,
    pub true_stages: Vec<LifecycleStage>,
    pub evidence: Vec<LifecycleEvidence>,
    pub missing: Vec<LifecycleStage>,
}
```

`missing` is the complement of `true_stages` over the full ordered sequence. `next_missing()` returns the earliest gap — the immediate lifecycle blocker.

### 3.2 bounds

The bounds module enforces the PDDL8 eight-bound constraint at the lifecycle level. The primary check is **Need9**: a work package with more than 8 tasks violates the bound and must be split before projection.

```rust
pub const MAX_WORK_UNIT_TASKS: usize = 8;

pub fn check_work_unit(name: &str, task_count: usize) -> Option<BoundViolation>
```

The bound is the same constant family as bcinr-pddl (`PDDL8_MAX_CONJUNCTS = 8`, `PDDL8_MAX_PARAMS = 8`). This is not coincidental. The lifecycle domain actions are designed to have ≤ 8 preconditions and ≤ 8 effects. If a lifecycle action needed 9 preconditions, it would be a signal that the action is doing too much — it should be decomposed. The eight-bound is not an implementation limit; it is a *design constraint* that forces decomposition at the right level.

### 3.3 projection

The projection module performs the central transformation of bcinr-pddl-lsp: **lifecycle facts → PDDL8**.

The generated lifecycle domain contains 10 actions, each mapping to one lifecycle transition:

```
create_prd          : intent_captured(p) → prd_exists(p)
admit_prd           : prd_exists(p) → prd_admitted(p)
derive_ard          : prd_admitted(p) → ard_exists(p)
admit_ard           : ard_exists(p) → ard_admitted(p)
generate_work_units : ard_admitted(p) → work_units_generated(p)
implement_work_units: work_units_generated(p) → implementation_complete(p)
run_tests           : implementation_complete(p) → tests_passed(p)
project_docs        : tests_passed(p) → docs_projected(p)
prepare_release     : docs_projected(p) ∧ tests_passed(p) ∧ prd_admitted(p) ∧ ard_admitted(p)
                        → release_ready(p)
publish_release     : prd_admitted(p) ∧ ard_admitted(p) ∧ implementation_complete(p)
                      ∧ tests_passed(p) ∧ docs_projected(p) ∧ release_ready(p)
                        → published(p)
```

Every action satisfies PDDL8 bounds: at most 1 parameter, at most 6 preconditions, at most 1 add effect. The `publish_release` action has the most preconditions (6) — well within the 8-bound.

The generated problem maps `true_stages` to PDDL8 init atoms:

```pddl
(:init
  (intent_captured myproject)
  (prd_exists myproject)
  (prd_admitted myproject))
(:goal (published myproject))
```

The goal is always `published(project)`. The plan is always the shortest path from current lifecycle state to publish.

One critical implementation detail: the project name is sanitized to a PDDL-safe identifier. PDDL identifiers must start with a letter; temp directories and git branches can start with `.` or `-`. The sanitizer strips dots, maps underscores to hyphens, and prepends `p-` if the result does not start with a letter. This was discovered via a test failure when TempDir generated a project name starting with `.tmp` — a concrete example of the value of running through the full stack in tests.

### 3.4 planner_client

The planner client is a thin wrapper around bcinr-pddl. It calls the full stack:

```
domain_from_pddl(domain_text)
problem_from_pddl(problem_text)
GroundProblem::build(domain, problem)
GroundProblem::find_plan()  →  Pddl8Tape
execute_tape(tape, initial_state, goal, case_id, &[])  →  (log, receipt, OCEL)
```

No planning logic lives here. The planner client owns the protocol between bcinr-pddl-lsp and bcinr-pddl; it does not own the algorithms.

The `case_id` is constructed from the project name: `"lsp-{project_name}"`. This identifies the execution in the OCEL log and allows receipts from different executions of the same project to be distinguished by their OCEL case objects.

The error taxonomy maps bcinr-pddl errors forward to the diagnostics layer:

```rust
pub enum PlannerError {
    ParseError(String),      // PDDL parse error → PDDL_PARSE_ERROR
    GroundingError(String),  // grounding failed → EMPTY_GROUNDING
    NoAdmittedPlan,          // BFS exhausted → NO_ADMITTED_PLAN
    ExecutionError(String),  // Prolog8 denied step → STEP_DENIED
}
```

### 3.5 publish_gate

The publish gate is the final lifecycle check. It answers the question: *is this project in a state where publish is admitted?*

Five statuses:

| Status | Meaning |
|---|---|
| `OPEN` | No lifecycle information yet |
| `PARTIAL` | Required lifecycle stages complete but no plan executed |
| `BLOCKED` | Required stages missing; named in `blockers` |
| `ADMITTED` | bcinr-pddl executed the tape with `goal_reached = true` |
| `PUBLISHED` | Receipt with `goal_reached = true` exists on disk |

The gate is computed in two passes:

1. **`from_lifecycle()`** — checks that all six required lifecycle stages are present (`prd_admitted`, `ard_admitted`, `implementation_complete`, `tests_passed`, `docs_projected`, `release_ready`). Returns `BLOCKED` with named missing stages, or `PARTIAL` if all six are present.

2. **`from_plan_result()`** — elevates `PARTIAL` to `ADMITTED` only if bcinr-pddl returned `goal_reached = true` in the receipt. A plan that terminated without reaching the goal cannot elevate the gate.

This two-pass design is important: the gate is `ADMITTED` only when the Prolog8 execution actually proved that `published(project)` holds in the final state. Lifecycle stage presence alone is necessary but not sufficient.

### 3.6 virtual_docs

The virtual document registry exposes ten `bcinr-pddl://` URIs:

| URI | Content |
|---|---|
| `bcinr-pddl://project/lifecycle` | Full lifecycle state as JSON |
| `bcinr-pddl://project/status` | Summary: stage count, next step, gate status |
| `bcinr-pddl://pddl/domain` | Generated PDDL8 domain text |
| `bcinr-pddl://pddl/problem` | Generated PDDL8 problem text |
| `bcinr-pddl://pddl/plan` | Plan steps + goal_reached |
| `bcinr-pddl://pddl/tape` | Pddl8Tape (alias for plan) |
| `bcinr-pddl://execution/log` | Step-by-step execution log with per-step receipt hashes |
| `bcinr-pddl://ocel/events` | OCEL event log |
| `bcinr-pddl://receipt/latest` | Pddl8ExecutionReceipt |
| `bcinr-pddl://publish/gate` | Gate status, blockers, admitted flag |

These documents are the project map. An agent that opens `bcinr-pddl://project/status` learns the current lifecycle position, the next step, and the gate status in a single JSON response. An agent that opens `bcinr-pddl://receipt/latest` gets the BLAKE3 chain hash, plan root, state root, goal root, and goal_reached — everything needed to verify that execution occurred lawfully.

The documents are computed on demand from the cached plan result. If no plan has been executed yet, unresolved documents return `{"status":"CANDIDATE"}` — a bounded status, not an error. This follows the lsp-max law: no victory language, no silent unknowns. The status is `CANDIDATE` until a receipt makes it `ADMITTED`.

### 3.7 diagnostics

The diagnostics module maintains the full taxonomy of lifecycle and planning diagnostic codes:

**Lifecycle diagnostics (from missing stages):**
```
INTENT_MISSING        ERROR    no intent file found
PRD_MISSING           ERROR    no docs/prd.md
PRD_NOT_ADMITTED      WARNING  PRD exists but lacks ADMITTED marker
ARD_MISSING           ERROR    no docs/ard.md
ARD_NOT_ADMITTED      WARNING  ARD exists but lacks ADMITTED marker
WORK_UNITS_MISSING    INFO     no work-units file
IMPLEMENTATION_INCOMPLETE INFO  no source files
TESTS_NOT_PASSED      WARNING  no passing test report
DOCS_NOT_PROJECTED    INFO     no projected docs beyond PRD/ARD
RELEASE_NOT_READY     INFO     no release artifact
PUBLISH_BLOCKED       WARNING  publish goal not reached
```

**Bounds diagnostics (from bounds module):**
```
WORK_UNIT_NEED9            ERROR    work package > 8 tasks
ACTION_PARAMETER_OVERFLOW  WARNING  action > 8 parameters
ACTION_PRECONDITION_OVERFLOW WARNING action > 8 preconditions
```

**Planner diagnostics (from planner_client):**
```
PDDL_PARSE_ERROR      ERROR  generated PDDL8 failed to parse
EMPTY_GROUNDING       ERROR  no ground actions produced
NO_ADMITTED_PLAN      ERROR  BFS exhausted without reaching goal
STEP_DENIED           ERROR  Prolog8 denied a tape step
```

Severity mapping is intentional. `PRD_MISSING` is an ERROR because no lifecycle can proceed without a PRD. `DOCS_NOT_PROJECTED` is INFO because it is a later-stage concern. `TESTS_NOT_PASSED` is a WARNING because it is a firm gate requirement but not a show-stopper for early lifecycle work.

### 3.8 code_actions

Code actions offer concrete repair for each diagnostic:

| Diagnostic | Action |
|---|---|
| `PRD_MISSING` | Create PRD skeleton at `docs/prd.md` |
| `ARD_MISSING` | Derive ARD from admitted PRD |
| `NO_ADMITTED_PLAN` | Run bcinr-pddl plan |
| `WORK_UNIT_NEED9` | Split work package into ≤8 tasks |
| `PUBLISH_BLOCKED` | Open receipt + explain publish gate |

Actions do not mutate files directly — they return skeleton templates or invoke named commands that the client can apply. This follows the lsp-max read-only LSP surface law: the LSP emits intents, it does not execute them unilaterally.

---

## 4. The Full Data Flow

A single call to `analyze_and_publish(uri)` executes the full stack:

```
1. lifecycle::scan(workspace_root)
     → ProjectLifecycle { true_stages, missing, evidence }

2. bounds::check_lifecycle_domain()
     → Vec<BoundViolation>  (empty for well-formed lifecycle domain)

3. projection::project(&lifecycle)
     → Pddl8Projection { domain_text, problem_text }

4. planner_client::plan_and_execute(&projection, case_id)
     → Result<PlanResult { plan_steps, log, receipt, ocel }, PlannerError>

5. publish_gate::from_plan_result(&lifecycle, &result)
     → PublishGate { status, blockers }

6. plan_cache updated with projection + result + gate

7. diagnostics::lifecycle_diagnostics(&lifecycle)
   + diagnostics::bound_diagnostics(&violations)
     → Vec<Diagnostic>

8. client.publish_diagnostics(uri, diagnostics)

9. client.log_message(INFO, "publish gate = {status}. Next: {next_missing}")
```

This runs on every `didOpen`, `didChange`, and `didSave`, and on every `bcinrPddl.refreshLifecycle` command. The analysis is synchronous within the async handler — for the lifecycle sizes involved (tens of files, a 10-action domain, <20 objects) the full cycle completes in milliseconds.

---

## 5. The Agent Interface

The publish gate and virtual documents together form the agent interface. An agent operating in Claude Code or over MCP can:

**Query current lifecycle position:**
```
workspace/executeCommand bcinrPddl.openVirtualDocument
args: ["bcinr-pddl://project/status"]
→ {"project": "myproject", "next_step": "ard_exists", "publish_gate": "BLOCKED"}
```

**Request the next step:**
The next step is always `lifecycle.next_missing()` — the earliest lifecycle stage not yet evidenced. The agent does not need to understand the lifecycle graph; it only needs to act on the returned stage name and create the corresponding artifact.

**Verify execution:**
```
workspace/executeCommand bcinrPddl.openVirtualDocument
args: ["bcinr-pddl://receipt/latest"]
→ {"plan_root": "...", "chain_hash": "...", "goal_reached": true}
```

**Check publish gate:**
```
workspace/executeCommand bcinrPddl.explainPublishGate
→ {"status": "ADMITTED", "blockers": [], "admitted": true}
```

This interface is exactly what the BRCE framework demands: the agent does not decide what to do next based on its own reasoning. It reads the project map — which is mechanically derived from lifecycle facts and the PDDL8 plan — and acts on what it finds.

---

## 6. Relationship to the Full BRCE Stack

bcinr-pddl-lsp occupies the outermost ring of the BRCE stack:

```
wasm4pm (60 process-mining algorithms, OCEL foundation)
  ↑
wasm4pm-compat (canonical types: OCEL, PDDL8, conformance)
  ↑
prolog8 (bounded proof engine: Horn rules, NAF, BLAKE3 receipts)
  ↑
bcinr-pddl (bounded planning: STRIPS-8, BFS, Prolog8 gate, OCEL out)
  ↑
bcinr-pddl-lsp (lifecycle formalization, PDDL8 projection, LSP surface)
```

Each layer adds a bounded transformation:

- wasm4pm → process-mining algorithms over OCEL
- wasm4pm-compat → canonical type language for the stack
- prolog8 → `R ⊢ A` decidable over bounded Horn databases
- bcinr-pddl → `G_F^B` (candidate futures) + bounded execution + receipts
- bcinr-pddl-lsp → project lifecycle → `G_F^B` via PDDL8 projection

The LSP is not a peripheral component. It is the interface between the project as it exists (files, evidence, git state) and the formal BRCE machinery that decides whether publish is admitted.

Without bcinr-pddl-lsp, the BRCE machinery is a planner with no inputs. With it, the planner has a live, file-grounded projection of the project's current lifecycle state — and the LSP surface delivers the plan back to every client that can speak JSON-RPC.

---

## 7. The Lifecycle Domain as Process Model

The bcinr-lifecycle PDDL8 domain is itself a process model. It encodes the organization's theory of how a project moves from intent to publish. This has an important implication: the domain can be mined.

Every execution of `execute_tape()` for the lifecycle domain produces an OCEL trace. Over many projects, these traces accumulate into an OCEL log. The wasm4pm process-mining algorithms can be applied to this log to discover the *actual* lifecycle process model — the process as it was executed, not as it was specified.

This creates a feedback loop:

```
bcinr-pddl-lsp executes lifecycle plan
  → OCEL events (one per lifecycle step admitted)
  → accumulated OCEL log (many projects)
  → wasm4pm discovers actual process model
  → discovered model compared to specified domain
  → conformance score (fitness, precision)
  → deviations → lifecycle domain update
  → updated PDDL8 domain → better plans
```

The lifecycle domain is not fixed. It is a living process model that can be improved by mining the traces of its own executions. bcinr-pddl-lsp is both a planning surface and an event producer for the process-intelligence loop that closes around it.

---

## 8. Falsification Properties

The test suite establishes falsifiability for every core claim. Key falsification pairs:

**Claim: `prd_admitted` requires the ADMITTED marker, not just file presence.**
- Proof: `prd_without_admitted_marker_gives_prd_exists_not_admitted` passes with candidate marker.
- Falsification: remove `ADMITTED` from admitted PRD → `PrdAdmitted` disappears.

**Claim: `published` requires a receipt with `goal_reached: true`, not just a receipt file.**
- Proof: `published_not_triggered_by_false_goal_reached` asserts `!lc.has(Published)`.
- Falsification: change to `goal_reached: true` → `Published` appears.

**Claim: The lifecycle domain parses through bcinr-pddl without modification.**
- Proof: `domain_parses_through_bcinr_pddl` calls `domain_from_pddl(emit_domain())` and asserts 10 actions.
- Falsification: corrupt the domain text → parse error.

**Claim: Need9 is detected at 9 tasks and not at 8.**
- Proof: `work_unit_within_bound_has_no_violation` (8 tasks → None) and `work_unit_exceeding_bound_is_need9` (9 tasks → Some).
- Falsification: change threshold to 7 → 8-task unit triggers Need9.

**Claim: PDDL name sanitization prevents parser failures on non-letter-initial names.**
- Discovered via test failure: TempDir produces `.tmp...` prefix → PDDL parse error.
- Fixed: prepend `p-` when name does not start with an ASCII letter.
- Falsification: revert sanitization → test with TempDir fails.

**Claim: publish gate is `BLOCKED` for empty projects.**
- Proof: `empty_project_publish_gate_is_blocked` asserts `status_label() == "BLOCKED"`.
- Falsification: fill all required stages → gate becomes `PARTIAL`.

---

## 9. What the LSP Cannot Do

bcinr-pddl-lsp cannot execute project-changing commands without admission. It emits templates, not mutations. The `createPrd` command returns a PRD skeleton string; the client must apply it. The `runPlan` command triggers analysis; the client must observe the resulting diagnostics.

bcinr-pddl-lsp cannot reason about project semantics. It cannot evaluate whether a PRD is *good*, only whether a PRD *exists and contains the ADMITTED marker*. The ADMITTED marker is itself a weak signal — it is a convention, not a proof. A more rigorous future extension would gate `prd_admitted` on an Open Ontologies graph query that verifies the PRD's claims against a semantic model. That extension is not in scope.

bcinr-pddl-lsp cannot act without a workspace root. The current implementation requires an LSP `initialize` with a `root_uri` or `workspace_folders`. Rootless operation is not supported.

bcinr-pddl-lsp currently enforces the lifecycle in terms of file evidence, not semantic content. An empty `docs/prd.md` file with the string `ADMITTED` anywhere in it is treated as an admitted PRD. This is intentionally simple — the lifecycle scanner is a starting point, not a semantic validator. The architecture supports plugging in a richer scanner that calls an Open Ontologies client or an ontology-grounded admission function without changing the projection or planning layers.

---

## 10. Conclusion

bcinr-pddl-lsp is the answer to a question that classical language servers do not ask: *is this project in a state where the next lawful action can be derived mechanically?*

It answers that question by projecting project lifecycle state into bounded PDDL8, calling bcinr-pddl to find the shortest path to `published(project)`, and exposing everything — plan, gate, receipt, OCEL — through the LSP protocol.

The architecture encodes three properties:

**Grounded.** Lifecycle facts come from files on disk, not from agent assertions. An agent cannot advance the lifecycle by claiming; it must produce artifacts.

**Bounded.** Every computation terminates. The lifecycle domain has 10 actions. The BFS is depth-capped at 64. The Prolog8 policy gate is O(1) per step. The BLAKE3 chain is O(n) in the number of steps. Total wall clock: milliseconds per analysis cycle.

**Receipted.** The publish gate is `ADMITTED` only when bcinr-pddl has executed the lifecycle tape through Prolog8 admission and returned `goal_reached = true` with a non-empty BLAKE3 chain hash. Confidence does not admit publish. Receipts do.

The LSP is the lifecycle map. bcinr-pddl is the mechanical planner. Prolog8 is the admission gate. OCEL is what happened. BLAKE3 is the receipt. Publish is only admitted when the lifecycle plan reaches its goal with proof.

---

## Appendix A: Diagnostic Code Taxonomy

```
Lifecycle (from missing stages):
  INTENT_MISSING              ERROR
  PRD_MISSING                 ERROR
  PRD_NOT_ADMITTED            WARNING
  ARD_MISSING                 ERROR
  ARD_NOT_ADMITTED            WARNING
  WORK_UNITS_MISSING          INFO
  IMPLEMENTATION_INCOMPLETE   INFO
  TESTS_NOT_PASSED            WARNING
  DOCS_NOT_PROJECTED          INFO
  RELEASE_NOT_READY           INFO
  PUBLISH_BLOCKED             WARNING

Bounds:
  WORK_UNIT_NEED9             ERROR
  ACTION_PARAMETER_OVERFLOW   WARNING
  ACTION_PRECONDITION_OVERFLOW WARNING
  PREDICATE_ARITY_OVERFLOW    WARNING

Planner (from bcinr-pddl):
  PDDL_PARSE_ERROR            ERROR    ← ParseError
  EMPTY_GROUNDING             ERROR    ← GroundingError
  NO_ADMITTED_PLAN            ERROR    ← NoAdmittedPlan
  STEP_DENIED                 ERROR    ← ExecutionError
```

## Appendix B: Lifecycle Domain Action Table

| Action | Parameters | Preconditions | Add Effects | Del Effects |
|---|---|---|---|---|
| `create_prd` | `?p` | `intent_captured(?p)` | `prd_exists(?p)` | — |
| `admit_prd` | `?p` | `prd_exists(?p)` | `prd_admitted(?p)` | — |
| `derive_ard` | `?p` | `prd_admitted(?p)` | `ard_exists(?p)` | — |
| `admit_ard` | `?p` | `ard_exists(?p)` | `ard_admitted(?p)` | — |
| `generate_work_units` | `?p` | `ard_admitted(?p)` | `work_units_generated(?p)` | — |
| `implement_work_units` | `?p` | `work_units_generated(?p)` | `implementation_complete(?p)` | — |
| `run_tests` | `?p` | `implementation_complete(?p)` | `tests_passed(?p)` | — |
| `project_docs` | `?p` | `tests_passed(?p)` | `docs_projected(?p)` | — |
| `prepare_release` | `?p` | 4 preconditions | `release_ready(?p)` | — |
| `publish_release` | `?p` | 6 preconditions | `published(?p)` | — |

All actions: 1 parameter, ≤ 6 preconditions, 1 add effect, 0 delete effects. Every bound satisfied.

## Appendix C: Virtual Document URI Table

| URI | Module | Cached? |
|---|---|---|
| `bcinr-pddl://project/lifecycle` | lifecycle | No (recomputed) |
| `bcinr-pddl://project/status` | lifecycle + gate | Gate cached |
| `bcinr-pddl://pddl/domain` | projection | Yes |
| `bcinr-pddl://pddl/problem` | projection | Yes |
| `bcinr-pddl://pddl/plan` | planner_client | Yes |
| `bcinr-pddl://pddl/tape` | planner_client | Yes (alias) |
| `bcinr-pddl://execution/log` | planner_client | Yes |
| `bcinr-pddl://ocel/events` | planner_client | Yes |
| `bcinr-pddl://receipt/latest` | planner_client | Yes |
| `bcinr-pddl://publish/gate` | publish_gate | Yes |

## Appendix D: BRCE Formal Correspondence

| BRCE term | bcinr-pddl-lsp implementation |
|---|---|
| `O` (observations) | `ProjectLifecycle::true_stages` (from disk evidence) |
| `O*` (enriched observations) | OCEL events from lifecycle tape execution |
| `G_F^B` (bounded future graph) | `Pddl8Tape` from BFS over lifecycle domain |
| `μ_B` (admission morphism) | Prolog8 `may_fire/1` in `execute_tape` |
| `A` (admitted actions) | Lifecycle steps with `admitted = true` in execution log |
| `ρ` (receipting function) | BLAKE3 chain in `execute_tape` |
| `R` (receipt) | `Pddl8ExecutionReceipt` in `bcinr-pddl://receipt/latest` |
| `actuate(publish) ⟺ R ⊢ A` | Gate = `ADMITTED` only when `goal_reached = true` in receipt |
