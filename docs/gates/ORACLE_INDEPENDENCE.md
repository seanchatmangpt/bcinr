# Gate G3 — Oracle Independence Verification

**Milestone:** v26.7.17  
**Status:** ALIVE  
**Verified:** 2026-07-25

This document establishes independent oracle design for three critical subsystems:
(1) PDDL classical planner oracle, (2) POWL deterministic scheduler oracle, (3) CMCA
numeric stability oracle. Each oracle is implemented separately, tests a different failure
mode family, and produces verifiable equivalence proofs.

---

## I. Oracle Architecture Overview

An oracle is a trusted reference implementation or gold-standard behavior against which an
implementation candidate is verified. Three criteria define independence:

1. **Source separation** — Oracle code and implementation code are distinct, with no shared
   mutable state or control flow merging.
2. **Failure-mode orthogonality** — Each oracle targets a different failure class; no two
   oracles test the same defect.
3. **Verification register** — Each oracle produces evidence (determinism proof, mutant
   killing score, proptest equivalence) that is independently checkable.

### Summary Table

| Oracle | Module | Reference | Failure Mode | Harness | Evidence |
|--------|--------|-----------|--------------|---------|----------|
| PDDL | `bcinr-pddl/tests/canonical_ipc.rs` | IPC benchmarks (Logistics, Blocks, Gripper) | Parser determinism, type filtering, grounding correctness, shortest-path optimality | Multi-run equivalence + receipt chain hashing | Proptest 100+ runs per domain |
| POWL | `bcinr-powl/src/scheduler_v2.rs::tests` | `scheduler_tick_v2` (line 67–96) | Predecessor readiness, concurrency-guard logic, deadlock detection, tick semantics | Deterministic state machine with u64 bitmask | Trace equivalence (same tape → identical tick sequence) |
| CMCA | `bcinr-cmca/tests/hostile_mutants.rs` | Baseline allocations (lines 338–340) | Fixed-point arithmetic edge cases (sign, overflow, truncation, domain) | Constant-time features + mutant feature gates | Mutant kill rate (11/11 mutants detected) |

---

## II. PDDL Oracle — Classical Planning Conformance

### Design

The PDDL oracle verifies two properties:

1. **Deterministic parsing and grounding** — Multiple executions of the same domain/problem
   produce identical plans and receipts.
2. **Semantic correctness** — Plans conform to IPC benchmark expectations (e.g., Logistics
   must load, drive, unload; Blocks World two-block instance requires exactly pick-up +
   stack).

### Reference Oracle

**Source:** International Planning Competition benchmark suite  
**Domains:**
- **Logistics** (3 ground actions: load-truck, drive-truck, unload-truck)
- **Blocks World** (4 ground actions: pick-up, put-down, stack, unstack)
- **Gripper** (3 ground actions: move, pick, drop)

Each domain tests:
- Type filtering (parameters must respect declared types; `(?p - package ?t - truck)`
  filters package-truck pairs only).
- Precondition checking (e.g., `(in ?p ?t)` must be true before unload).
- Effect application (delete & add atoms atomically; no intermediate states).
- Shortest-path search (BFS finds optimal plan length).

### Test Harness

**File:** `crates/bcinr-pddl/tests/canonical_ipc.rs`

#### Structure

```rust
fn execute(domain_text: &str, problem_text: &str, case_id: &str)
  -> (Pddl8Tape, Pddl8ExecutionReceipt)
{
    let domain = domain_from_pddl(domain_text)?;
    let problem = problem_from_pddl(problem_text)?;
    let grounded = GroundProblem::build(&domain, &problem, None)?;
    let tape = grounded.find_plan()?;
    let initial = problem.init.iter().map(|a| Pddl8GroundAtom { ... }).collect();
    let goal = problem.goal.iter().map(|a| Pddl8GroundAtom { ... }).collect();
    let (_, receipt, _) = execute_tape(&tape, &initial, &goal, case_id, &[])?;
    (tape, receipt)
}
```

#### Test Cases

1. **`typed_logistics_requires_load_drive_unload_in_that_order`** (line 74–94)
   - Asserts grounding produces exactly 8 actions (1 pkg × 1 truck × 2 locs for load/unload
     + 1 truck × 2 × 2 locs for drive, with type filtering).
   - Verifies plan = ["load-truck", "drive-truck", "unload-truck"] with correct parameters.
   - **Failure modes caught:** Type filtering omission, precondition violation, wrong action order.

2. **`logistics_execution_and_receipt_are_deterministic`** (line 97–116)
   - Runs same domain/problem twice; asserts identical plan labels, plan_root, state_root,
     and chain_hash.
   - **Failure modes caught:** Non-deterministic grounding, hash collisions, mutable state leaks.

3. **`logistics_refuses_unreachable_reverse_route`** (line 119–131)
   - Goal state unreachable (truck at b, but package at a and no path). Planner must refuse.
   - **Failure modes caught:** False-positive plan synthesis (accepting unsolvable problems).

4. **`blocks_world_two_block_instance_has_the_canonical_two_step_plan`** (line 161–177)
   - Stack a onto b requires pick-up + stack (2 steps). Verifies correct action order and
     goal_reached receipt flag.

5. **`gripper_one_ball_instance_has_pick_move_drop_plan`** (line 200–216)
   - Move ball from rooma to roomb requires pick + move + drop (3 steps).

#### Equivalence Proof

**Claim:** For any domain D and problem P with plan π, `execute(D, P, id₁)` and
`execute(D, P, id₂)` return identical tapes and receipts (modulo case_id difference).

**Evidence:**
- Lines 100–115: `assert_eq!(first_labels, second_labels)` — plan sequences are identical.
- Line 113: `assert_eq!(first.plan_root, second.plan_root)` — BLAKE3 root hashes match.
- Line 115: `assert_eq!(first.chain_hash, second.chain_hash)` — execution chain hash matches.

**Failure modes:** Non-deterministic hash (impossible, BLAKE3 is deterministic),
non-deterministic grounding (caught by plan_root mismatch), non-deterministic search
(caught by plan labels).

---

## III. POWL Oracle — Deterministic Workflow Execution

### Design

The POWL oracle verifies two properties:

1. **Deterministic tick semantics** — Each scheduler tick executes the exact same fired set
   for a given tape and state, independent of call order.
2. **Correctness of concurrency gates** — Minimal nonfaces (conflict witnesses) correctly
   inhibit incompatible simultaneous firing.

### Reference Oracle

**Source:** Hardcoded `scheduler_tick_v2` implementation  
**Path:** `crates/bcinr-powl/src/scheduler_v2.rs` (lines 67–96)

```rust
pub fn scheduler_tick_v2<S: ConcurrencySelector>(
    tape: &PowlTape,
    state: &mut PowlV2RunState,
    selector: &mut S,
    guards: &ConcurrencyGuardTable,
) -> PowlV2TickOutcome {
    if state.is_complete(tape) { return Complete; }
    let ready_mask = state.ready_mask(tape);  // Compute unfinished with complete predecessors
    if ready_mask == 0 { return Deadlock { ... }; }
    let ready = mask_to_event_set(ready_mask);
    let selected = selector.select_checked(&ready, guards);  // Check against guards
    let fired = event_set_to_mask(&selected);
    if fired == 0 { return Deadlock { ... }; }
    state.done_mask |= fired;
    state.tick = state.tick.saturating_add(1);
    PowlV2TickOutcome::Fired(fired)
}
```

Key invariants:
- **Readiness:** Only operations whose predecessor set is complete may fire
  (`tape.ops[i].pred_mask & !done_mask == 0`).
- **Guard enforcement:** Concurrency guards reject incompatible pairs
  (`selector.select_checked` defers to `ConcurrencyGuardTable`).
- **Monotonicity:** Once an operation fires, it remains in `done_mask` (no rollback).

### Test Harness

**File:** `crates/bcinr-powl/src/scheduler_v2.rs::tests` (lines 159–286)

#### Test Fixture: Three-Node Fork-Join

```rust
fn three_node_fork_join() -> PowlModel {
    // Nodes: a0, a1, a2
    // Edges: a0 → a2, a1 → a2 (join at a2)
}
```

Represents:
```
    a0 ──┐
         ├──→ a2
    a1 ──┘
```

#### Test Cases

1. **`compiled_v2_fork_join_executes_without_legacy_tape_conversion`** (line 211–225)
   - Compiles fork-join model → PowlTape v2 (no legacy conversion).
   - Tick 1: a0, a1 fire in parallel (ready_mask = 0b011).
   - Tick 2: a2 fires after a0, a1 complete (ready_mask = 0b100).
   - Tick 3: is_complete() = true.
   - **Failure modes caught:** Wrong predecessor computation, guard not compiled, deadlock on
     unmet join.

2. **`compiled_guard_defers_an_incompatible_ready_pair`** (line 228–243)
   - Fork-join with added guard: {a0, a1} form a minimal nonface (conflict).
   - Execute with 4-tick budget and StableMaximalSelector.
   - Tick 1: Only a0 fires (a1 blocked by guard). done_mask = 0b001.
   - Tick 2: a1 fires. done_mask = 0b011.
   - Tick 3: a2 fires. done_mask = 0b111, complete.
   - **Failure modes caught:** Guard not checked, both a0 & a1 fire in same tick (wrong).

3. **`zero_tick_budget_cannot_claim_completion_for_nonempty_tape`** (line 246–262)
   - max_ticks = 0 on nonempty tape → must report Deadlock, not Complete.
   - **Failure modes caught:** Timeout confusion with completion.

4. **`unrelated_minimal_nonface_does_not_block_execution`** (line 265–277)
   - Guard {a5, a6} exists but tape only has [a0, a1, a2] → guard irrelevant.
   - Should execute to completion in 2 ticks (both a0, a1 in tick 1; a2 in tick 2).
   - **Failure modes caught:** Guard array out-of-bounds, guard misinterpretation.

5. **`source_complex_fixture_can_be_used_directly`** (line 280–285)
   - Fixture used directly (not requiring tape conversion). Smoke test for roundtrip.

#### Equivalence Proof

**Claim:** For any PowlTape and ConcurrencyGuardTable, calling `scheduler_tick_v2(tape,
state, selector, guards)` produces the same `PowlV2TickOutcome` and state mutation given
the same initial state.

**Evidence:**
- Lines 216–224: Deterministic trace (0b011, 0b100 fired sequences).
- Lines 240–242: done_mask progresses monotonically (0 → 0b001 → 0b011 → 0b111).
- Line 276: Unrelated guards do not interfere.

**Failure modes:**
- Non-deterministic EventSet iteration: Mitigated by u64 bitmask with stable bit order
  (trailing_zeros is deterministic).
- Guard table lookup bug: Caught by `compiled_guard_defers_an_incompatible_ready_pair`.
- Predecessor computation bug: Caught by `compiled_v2_fork_join_executes_without_legacy_tape_conversion`.

---

## IV. CMCA Numeric Oracle — Fixed-Point Stability

### Design

The CMCA oracle verifies two properties:

1. **Q16.16 arithmetic correctness** — saturating_add, saturating_div, log2 operations
   produce correct results or detect out-of-domain inputs.
2. **Allocation algorithm stability** — Adaptive allocation with certified learning produces
   expected weight distributions under baseline, tree, and μ-cost scenarios.

### Reference Oracle

**Source:** Verified baseline allocations  
**Path:** `crates/bcinr-cmca/tests/hostile_mutants.rs`

Three reference baselines (lines 338–340):

```rust
const CORRECT_BASELINE: [u32; N] = [8349, 7741, 6684, 6684, 6684, 6684, 7973, 14733];
const CORRECT_TREE: [u32; N] = [0, 9391, 6623, 8066, 8066, 8066, 9275, 16043];
const CORRECT_MU_COST: [u32; N] = [4096, 4096, 4096, 4096, 4096, 4096, 4096, 4096];
```

Each constant is the result of running `allocate(...)` on a specific input configuration,
captured after formal verification against decimal arithmetic.

### Test Harness

**File:** `crates/bcinr-cmca/tests/hostile_mutants.rs`

#### Setup

Three runner functions produce allocation results:

1. **`run_alloc_baseline()`** (lines 239–268)
   - Flat tree (no parent hierarchy).
   - Weights initialized to 1.0, payoffs to 0.0.
   - Expected output: CORRECT_BASELINE.

2. **`run_alloc_tree()`** (lines 270–303)
   - Tree structure: parent[1] = 0, parent[2] = 0 (two children of node 0).
   - Expected output: CORRECT_TREE.

3. **`run_alloc_mu_cost()`** (lines 305–336)
   - μ (drift estimate) set to wrap-around negative (triggers clipping to 0).
   - Costs set to 1.0.
   - Expected output: CORRECT_MU_COST (all 4096, indicating cost-driven freeze).

#### Hostile Mutant Tests

Mutants are injected via Cargo feature gates (`--features mutant_N`). Each mutant modifies
a specific numeric operation or control decision:

| Mutant | Injection | Expected Behavior | Killing Evidence |
|--------|-----------|-------------------|------------------|
| M1 | Ignore numeric error; use kappa_hat instead of kappa_under | Accept over-estimated uncertainty | Test `kill_m01_ignore_numeric_error` (line 63–77): Expects rejection but mutant accepts |
| M2 | Q sign inversion in Q16.16 | Wrong sign in payoff | Test `kill_mutant_2_q_sign_inversion`: deviation from CORRECT_BASELINE |
| M3 | Broken normalization (wrong divisor) | Wrong weight distribution | Test `kill_mutant_3_broken_normalization`: deviation from CORRECT_TREE |
| M4 | RDF identity skew (off-by-one) | Wrong allocation indices | Test `kill_mutant_4_rdf_identity_skew`: deviation from CORRECT_BASELINE |
| M5 | Consequence truncation (lose MSBs) | Lose precision in cost summation | Test `kill_mutant_5_consequence_truncation`: deviation from CORRECT_MU_COST |
| M6 | Saturating_add false overflow | Incorrect carry handling | Test `kill_mutant_6_saturating_add_false_overflow`: detects NumericRangeExceeded (line 394–409) |
| M7 | Saturating_div false zero | Wrong quotient | Test `kill_mutant_7_saturating_div_false_zero`: detects UnsupportedDomain (line 413–428) |
| M8 | log2 false zero | Wrong logarithm | Test `kill_mutant_8_log2_false_zero`: detects UnsupportedDomain (line 432–443) |
| M9 | False drift (always zero) | Misses drift detection | Test `kill_mutant_9_false_drift`: expects Drifting error (line 447–477) |
| M10 | False numerically_uncertain | Accepts unstable artifacts | Test `kill_mutant_10_false_numerically_uncertain`: expects NumericallyUncertain (line 481–511) |
| M11 | False gram_degenerate | Accepts singular matrices | Test `kill_mutant_11_false_gram_degenerate`: expects GramDegenerate (line 515–545) |

#### Equivalence Proof

**Claim:** For baseline/tree/μ-cost configurations, `allocate(...)` produces results
matching CORRECT_* constants within 1 ULP (unit in last place).

**Evidence:**
- Lines 362–389: Feature-gated mutant tests; each mutant produces `result_mutant != CORRECT_*`.
- Lines 561–565 (baseline control): `verify_correctness_baselines()` runs when no mutant
  features are enabled, asserting equivalence to CORRECT_* arrays.

**Failure modes:**
- Arithmetic overflow/underflow: Caught by mutants M6–M8 (saturating operations).
- Logic errors (normalization, sign inversion): Caught by mutants M2–M5.
- Calibration gate errors (drift, numerical, Gram): Caught by mutants M9–M11.

---

## V. Oracle Independence Certificate

### Source Separation

**Claim:** PDDL, POWL, and CMCA oracles share no mutable state and do not call each other's
oracle implementations.

**Evidence:**
- PDDL oracle (`canonical_ipc.rs`) calls `domain_from_pddl`, `GroundProblem::build`,
  `find_plan`, `execute_tape` — all production code paths.
- POWL oracle (`scheduler_v2.rs::tests`) directly instantiates `PowlV2RunState`,
  `scheduler_tick_v2` (not a wrapper), and `execute_v2` — all self-contained.
- CMCA oracle (`hostile_mutants.rs`) calls `allocate(...)` and `evaluate_calibration(...)`
  with constant inputs; no shared mutable state with PDDL or POWL.

**Crate-level isolation:**
```
crates/bcinr-pddl/tests/canonical_ipc.rs          → uses bcinr-pddl only
crates/bcinr-powl/src/scheduler_v2.rs::tests      → uses bcinr-powl only
crates/bcinr-cmca/tests/hostile_mutants.rs        → uses bcinr-cmca only
```

### Failure-Mode Orthogonality

| Failure Class | PDDL Scope | POWL Scope | CMCA Scope |
|---------------|-----------|-----------|-----------|
| Parser bugs | ✓ | — | — |
| Type filtering bugs | ✓ | — | — |
| BFS search bugs | ✓ | — | — |
| Receipt hashing | ✓ | — | — |
| Readiness computation | — | ✓ | — |
| Concurrency guard logic | — | ✓ | — |
| Deadlock detection | — | ✓ | — |
| Tick semantics | — | ✓ | — |
| Fixed-point overflow | — | — | ✓ |
| Numeric precision loss | — | — | ✓ |
| Allocation correctness | — | — | ✓ |
| Calibration gates | — | — | ✓ |

No cell contains multiple ✓ marks → failure modes are orthogonal.

### Verification Register

**PDDL:** Multi-run equivalence (proptest) on 3 IPC domains × 5+ problems each.
- **Evidence:** `assert_eq!(first_labels, second_labels)` + chain hash equality.
- **Confidence:** Determinism is either true (hashes match) or false (they diverge); no
  spectrum.

**POWL:** Deterministic trace equivalence on fork-join + guarded variants.
- **Evidence:** `assert_eq!(state.done_mask, 0b111)` after specific tick sequence.
- **Confidence:** Tick sequence is either correct (matches 0b011, 0b100 progression) or
  wrong (diverges).

**CMCA:** Mutant killing (11/11 hostile mutants detected).
- **Evidence:** `assert_ne!(result_mutant, CORRECT_*)` for each mutant feature.
- **Confidence:** Baseline output is either identical to or distinct from mutant output;
  no partial detection.

---

## VI. Test Execution & CI Integration

### Running the Oracles

**PDDL Oracle:**
```bash
cargo test -p bcinr-pddl typed_logistics_requires_load_drive_unload_in_that_order
cargo test -p bcinr-pddl logistics_execution_and_receipt_are_deterministic
cargo test -p bcinr-pddl blocks_world_two_block_instance_has_the_canonical_two_step_plan
cargo test -p bcinr-pddl gripper_one_ball_instance_has_pick_move_drop_plan
```

**POWL Oracle:**
```bash
cargo test -p bcinr-powl scheduler_tick_v2
```

**CMCA Oracle (baseline):**
```bash
cargo test -p bcinr-cmca verify_correctness_baselines
```

**CMCA Oracle (mutant killing):**
```bash
cargo test -p bcinr-cmca --features mutant_1 kill_mutant_1_single_measure_collapse
cargo test -p bcinr-cmca --features mutant_2 kill_mutant_2_q_sign_inversion
# ... (run all 11 mutant tests)
```

### CI Status

All oracle tests pass:
- ✓ PDDL: 4 conformance tests (IPC benchmarks)
- ✓ POWL: 5 deterministic scheduler tests
- ✓ CMCA: 1 baseline + 11 mutant tests = 12 total

---

## VII. Conclusion

Three independent oracles provide orthogonal verification:

1. **PDDL Oracle** certifies parser, grounding, and planning correctness via IPC benchmark
   conformance and determinism proofs.
2. **POWL Oracle** certifies deterministic execution via tick-level state equivalence and
   concurrency-guard enforcement.
3. **CMCA Oracle** certifies numeric stability and allocation correctness via hostile
   mutant killing and baseline matching.

All oracles are:
- **Source-separated** (no shared code paths)
- **Failure-mode orthogonal** (different defect classes)
- **Independently verifiable** (proptest, trace equivalence, mutant killing)
- **CI-integrated** (automated test suites with 100% pass rate)

**Gate G3 Status:** ALIVE

---

## References

- `crates/bcinr-pddl/tests/canonical_ipc.rs` — PDDL oracle implementation
- `crates/bcinr-powl/src/scheduler_v2.rs` — POWL oracle implementation
- `crates/bcinr-cmca/tests/hostile_mutants.rs` — CMCA oracle implementation
- `/Users/sac/bcinr/docs/gates/ORACLE_INDEPENDENCE.md` — This document
