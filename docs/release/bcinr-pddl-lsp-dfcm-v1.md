# Release: bcinr-pddl-lsp DfCM v1

**Tag:** `PROJECT_LIFECYCLE_DFCM_ADMITTED`  
**Commit:** `616f4385`  
**Date:** 2026-06-26  
**Status:** ADMITTED  

---

## What this is

A language server that turns project lifecycle planning into bounded, receipted, mechanical execution.

Not a prose planning assistant. Not a generic PDDL LSP. Not a new planner.

The lifecycle map from intent to publish. The agent does not decide what to do next from its own reasoning. It reads `bcinr-pddl://project/next-step` and acts on what the map says.

---

## The loop

```
Workspace Evidence
→ Lifecycle Facts          (lifecycle scanner, 12 stages)
→ PDDL8                    (15-action domain, projection module)
→ Candidate Plan           (bcinr-pddl BFS, projection mode)
→ Explicit Admission       (bcinrPddl.executeTape, admission mode)
→ BLAKE3 Receipt           (.bcinr/receipts/latest.json)
→ OCEL                     (.bcinr/ocel/latest.json)
→ Publish Gate             (OPEN/BLOCKED/CANDIDATE/PARTIAL/ADMITTED/PUBLISHED/REFUSED)
→ LSP Surfaces             (diagnostics, code actions, virtual docs, commands)
```

---

## Test results

**55 tests, 0 failures.**

| Suite | Tests | Status |
|---|---|---|
| `tests/lifecycle_integration.rs` | 20 | ALL PASS |
| `tests/falsification.rs` | 35 | ALL PASS |

The falsification suite covers all 10 DfCM acceptance criteria with explicit counterfactuals — each test names the specific change that would cause it to fail.

---

## Lifecycle domain

The generated PDDL8 domain has 15 actions in two families:

**Lifecycle actions (9):**
```
create_prd → admit_prd → derive_ard → admit_ard → record_adr
→ generate_work_units → project_docs → prepare_release → publish_release
```

**Build coordination actions (6):**
```
request_build_slot → acquire_build_slot → implement_work_units
→ run_tests → record_build_ocel → emit_receipt
```

**`publish_release` has exactly 8 preconditions** — at the Need9 boundary. If a 9th condition is needed, the action must be split into `prepare_release` + `finalize_publish`. The bound is the design constraint.

All actions: ≤1 parameter, ≤8 preconditions, ≤1 add effect. Every PDDL8 bound satisfied.

---

## Key invariants

**Candidate ≠ Admitted.**  
`didOpen`, `didChange`, `didSave`, `bcinrPddl.runPlan` → `projection_mode` only. Produces `PlanCandidate`. No receipt. No OCEL. Gate stays at PARTIAL at most.

`bcinrPddl.executeTape` only → `admission_mode`. Executes tape under Prolog8 `may_fire` gate. Emits BLAKE3 chain. Persists receipt and OCEL. Gate advances to ADMITTED iff `goal_reached = true`.

**Publish requires receipt.**  
`PublishGateStatus::Admitted` is only reachable from `from_plan_result()` with `receipt.goal_reached = true`. Lifecycle stage presence is necessary but not sufficient.

**Receipt integrity is checked.**  
`goal_reached: false` in `.bcinr/receipts/latest.json` → gate is `REFUSED`, not `PUBLISHED`. A receipt that says the goal was not reached is evidence of failure, not success.

**Direct heavy builds are blocked.**  
`cargo build`, `wasm-pack build`, `tsc`, `gradle`, `npm run build` without a broker slot → `DIRECT_HEAVY_COMMAND_BLOCKED` diagnostic. The build broker closes the agent-concurrency gap inside the lifecycle map — no separate semaphore daemon.

**Need9 means split.**  
Work packages with >8 tasks, actions with >8 preconditions, goals with >8 atoms: all blocked at the bound. The eight-bound is not an implementation limit; it is the design constraint that forces decomposition at the right level.

---

## Build broker

The broker is not a side utility. It is the proof that process is the coordination layer.

The prior agent-concurrency problem (memory spike from uncoordinated multi-agent builds) was never a Rust memory setting. It was:

```
uncoordinated agents → unadmitted heavy builds → resource collapse
```

Now:

```
request_build_slot → acquire_build_slot → heavy build → record_build_ocel → emit_receipt
```

This is the same BRCE mechanism as lifecycle admission. The coordination layer is the process model, not a semaphore.

Broker state is exposed at `bcinr-pddl://build/broker`. Slot denial emits OCEL. Slot release emits OCEL. Every heavy build that runs through the broker is traceable in the OCEL log.

---

## Virtual document surface

15 virtual documents:

| URI | Content |
|---|---|
| `bcinr-pddl://project/lifecycle` | Full lifecycle state JSON |
| `bcinr-pddl://project/status` | Summary: stage count, next step, gate |
| `bcinr-pddl://project/evidence` | File evidence per stage |
| `bcinr-pddl://project/next-step` | Next lawful action + blockers |
| `bcinr-pddl://bounds/report` | Bound violations |
| `bcinr-pddl://pddl/domain` | Generated PDDL8 domain |
| `bcinr-pddl://pddl/problem` | Generated PDDL8 problem |
| `bcinr-pddl://pddl/plan` | Candidate plan or admitted plan |
| `bcinr-pddl://pddl/tape` | Alias for plan |
| `bcinr-pddl://execution/log` | Step-by-step log with per-step receipt hashes |
| `bcinr-pddl://ocel/events` | OCEL trace |
| `bcinr-pddl://receipt/latest` | BLAKE3 receipt |
| `bcinr-pddl://publish/gate` | Gate status, blockers, admitted flag |
| `bcinr-pddl://build/broker` | Build slot state |
| `bcinr-pddl://agent/assignments` | Next lawful step for agent consumption |

An agent that opens `bcinr-pddl://agent/assignments` learns the next lawful lifecycle step without reasoning. It acts on what the map says.

---

## LSP commands

**Projection mode (safe, auto):**
- `bcinrPddl.refreshLifecycle`
- `bcinrPddl.runPlan`
- `bcinrPddl.generateProjection`

**Admission mode (explicit only):**
- `bcinrPddl.executeTape`

**Lifecycle repair:**
- `bcinrPddl.createPrd` / `createArd` / `createAdr`
- `bcinrPddl.generateWorkUnits`
- `bcinrPddl.splitNeed9`

**Gate and receipt:**
- `bcinrPddl.explainPublishGate`
- `bcinrPddl.verifyReceipt`
- `bcinrPddl.openVirtualDocument`

**Build broker:**
- `bcinrPddl.requestBuildSlot`
- `bcinrPddl.releaseBuildSlot`
- `bcinrPddl.wrapHeavyCommand`

**OCEL:**
- `bcinrPddl.emitOcelSnapshot`

---

## Diagnostic codes

| Code | Severity | Meaning |
|---|---|---|
| `INTENT_MISSING` | ERROR | No intent file |
| `PRD_MISSING` | ERROR | No PRD |
| `PRD_NOT_ADMITTED` | WARNING | PRD exists, no ADMITTED marker |
| `ARD_MISSING` | ERROR | No ARD |
| `ARD_NOT_ADMITTED` | WARNING | ARD exists, next action is `admit_ard` not `derive_ard` |
| `ADR_MISSING` | WARNING | No ADR in docs/adr/ |
| `WORK_UNIT_NEED9` | ERROR | Work package > 8 tasks |
| `TESTS_NOT_PASSED` | WARNING | No passing test report |
| `PUBLISH_BLOCKED` | WARNING | Published stage not reached |
| `BUILD_SLOT_DENIED` | ERROR | Second slot request while slot acquired |
| `DIRECT_HEAVY_COMMAND_BLOCKED` | ERROR | Heavy command without broker |
| `OCEL_TRACE_MISSING` | WARNING | No OCEL after admission |
| `RECEIPT_INTEGRITY_ERROR` | ERROR | Receipt file present but goal_reached=false |

---

## Storage layout

```
.bcinr/
├── receipts/latest.json        ← written by executeTape
├── ocel/latest.json            ← written by executeTape
├── test-report.json            ← lifecycle scanner reads this for tests_passed
├── release.json                ← lifecycle scanner reads this for release_ready
└── work-units.json             ← lifecycle scanner reads this for work_units_generated

docs/
├── prd.md                      ← must contain "ADMITTED" for prd_admitted
├── ard.md                      ← must contain "ADMITTED" for ard_admitted
├── adr/001-*.md                ← any .md here → adr_recorded
└── work-units.md               ← alternative to .bcinr/work-units.json
```

---

## Final law

```
actuate(publish) ⟺ R ⊢ published(project)
```

The project is not published when the code compiles.  
The project is published when:

- the lifecycle map exists,
- the PDDL8 projection exists,
- the candidate plan exists,
- the explicit admission receipt exists,
- the OCEL trace exists,
- the publish gate is ADMITTED,
- and every agent can query `bcinr-pddl://agent/assignments` for the next lawful step.
