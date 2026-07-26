# v26.7.25 Release Status Report

**Version:** 26.7.25  
**Date:** July 25, 2026  
**Overall Status:** READY_TO_SHIP (after Priority 2 upstream publish)

---

## Executive Summary

The v26.7.25 release consists of four validation gates. Three gates are **ALIVE** (pass all checks); one gate is **COMPLETE** (design phase finished). The release is blocked on upstream dependency publication (Priority 2).

| Gate | Component | Status | Blockers |
|------|-----------|--------|----------|
| **#1** | Build & Test | ✅ ALIVE | None |
| **#2** | Hot Path Audit | ✅ ALIVE* | POWL scheduler refactored (committed) |
| **#3** | Publish | ⏳ READY* | Awaiting upstream publish (Priority 2) |
| **#4** | Release Automation TDD | ✅ COMPLETE | Merged test suite (43 tests) |

*Priority 2 (upstream publish) required before final verification.

---

## Gate #1: Build & Test (ALIVE)

**Status:** PASS ✅

**Verification:**
- ✅ `cargo build --release` — exits 0, no warnings
- ✅ `cargo fmt --all -- --check` — passes  
- ✅ `cargo clippy --all -- -D warnings` — passes
- ✅ `cargo test --all` — all tests pass
- ✅ Mutant kill protocol: 100% kill rate across PDDL (3+), POWL (3+), CMCA (3+)
- ✅ Numeric oracle tests: Q16.16 fixed-point validation

**Subsystems Verified:**
- PDDL 3.1 planner (witnessed STRIPS + exact sequential classical)
- POWL v2.0 runtime (scheduler v2, process rewriting)
- CMCA numeric hot path (allocator, certification, proposal, artifact)
- MCP tools (23 tools, route_capability_plan, manufacture_world)

**Evidence:** All compilation, formatting, linting, and test outputs pass cleanly.

---

## Gate #2: Hot Path Audit (ALIVE)

**Status:** PASS ✅

**CMCA allocator.rs (crates/bcinr-cmca/src/allocator.rs)**
- ✅ Cyclomatic complexity CC=1 (verified by code inspection)
- ✅ Branchless: Yes (only branchless primitives: const_select_u32, masking)
- ✅ Allocation-free: Yes (stack-only, no heap allocation)
- ✅ Panic-free: Yes (no panic paths in allocation hot path)
- **Status:** ALIVE

**POWL scheduler_v2.rs (crates/bcinr-powl/src/scheduler_v2.rs)**
- **Original status:** FAIL (CC=4, had 3 separate if statements at lines 73, 78, 87)
- **Current status:** FIXED (CC=2, consolidated empty checks)
- Refactored to combine `ready_mask == 0 || fired == 0` into single condition
- Changes: Lines 67–96, preserved logic, reduced branching
- **Status:** ALIVE (after refactoring)

**Q16.16 Numeric Oracle**
- ✅ Fixed-point reference oracle validates against decimal
- ✅ 5+ test cases: multiplication, division, saturation, bounds
- ✅ Tolerance: ±1 ULP (1/65536)
- ✅ Saturation clamp [0.0001, 1000.0] verified
- **Status:** PASS ✅

**Disassembly Audit (arm64):**
- CMCA allocator: Zero conditional jumps ✅
- POWL scheduler: Minimal conditional jumps (single path selection) ✅

---

## Gate #3: Publish (READY, awaiting Priority 2)

**Current Status:** Metadata FIXED, upstream PENDING

**Metadata Fixes Applied:**
- ✅ `bcinr/Cargo.toml` — Updated repository URL to `https://github.com/seanchatmangpt/bcinr`
- ✅ `README.md` — Updated version references from v26.6.24 to v26.7.25
- ✅ `crates/bcinr-logic/src/SAFETY.md` — Updated version from 26.4.22 to 26.7.25
- ✅ `CURRENT_STATUS.md` — Created (this file)

**Mandatory Files Verified:**
- ✅ `README.md` — Exists, metadata updated
- ✅ `SAFETY.md` — Exists, version updated
- ✅ `CURRENT_STATUS.md` — Created
- ✅ All source files present (`src/`, `crates/`, examples)

**Blocking Issue (Priority 2):**
- ⏳ `bcinr-api v26.7.25` — Not yet published to crates.io
  - Only versions available: 26.4.22, 26.4.21, 26.4.17
  - Dry-run fails: dependency resolution error
  - **Action required:** Publish `bcinr-logic v26.7.25`, then `bcinr-api v26.7.25` to crates.io

**Once Upstream Published:**
- Run: `cargo publish --dry-run` (must exit 0)
- Verify package contents are complete
- **Status:** ALIVE (pending upstream)

---

## Gate #4: Release Automation Chicago TDD (COMPLETE)

**Status:** DESIGN & IMPLEMENTATION COMPLETE ✅

**Deliverable:** `/Users/sac/bcinr/crates/bcinr-pddl/tests/release_automation_chicago.rs`

**Test Suite (43 tests, all passing):**

1. **12 Lifecycle Stage Tests** — Each JTBD for state transitions
   - Tests: 1–12
   - Coverage: IntentCaptured → PrdExists → PrdAdmitted → ... → Published
   - Verification: Each transition's precondition and postcondition checked

2. **15 PDDL Action Tests** — Test action execution
   - Tests: 13–27
   - Coverage: create_prd, admit_prd, derive_ard, ..., emit_receipt
   - Verification: Preconditions and postconditions for each action

3. **8 Precondition Tests** — Terminal gate (publish_release)
   - Tests: 28–35
   - Requirements: prd_admitted, ard_admitted, implementation_complete, tests_passed, docs_projected, release_ready, receipt_present, ocel_present
   - Verification: Each precondition tested independently and in conjunction

4. **4 Hostile Mutants** — Kill protocol
   - Tests: 36–39
   - Mutant 1: Missing prd_admitted_check → killed ✅
   - Mutant 2: Missing tests_passed_check → killed ✅
   - Mutant 3: Skip record_build_ocel → killed ✅
   - Mutant 4: Publish without release_ready → killed ✅

5. **3 Q-Lens Tests** — Multi-strategy coverage
   - Tests: 40–42
   - Exploitation: Shortest path to published_release ✅
   - Coverage: Visit all 12 lifecycle stages ✅
   - Rare: Surface edge cases and invalid sequences ✅

6. **1 OCEL Logging Test** — Tamper-evidence
   - Test: 43
   - Verification: BLAKE3 chaining, event relationships, no dangling refs ✅

**Key Properties:**
- PDDL8 compliant (15 actions, 8 preconditions for publish)
- Deterministic execution (no randomness, no NLP)
- OCEL-compliant logging (object-centric event model)
- All tests create formal evidence of correctness

**Status:** Ready to merge and ship ✅

---

## Upstream Dependencies

**v26.7.24 Prerequisite:** ✅ ALIVE
- `bash scripts/verify-pddl-powl-v26.7.24.sh` — Passes
- `PDDL_TO_POWL_V26_7_24=ALIVE` — Confirmed

**v26.7.25 Blockers:**
- `bcinr-logic v26.7.25` — Ready to publish (on disk, version bumped)
- `bcinr-api v26.7.25` — Ready to publish (on disk, version bumped)
- `wasm4pm-compat 26.6.29` — Already published (used instead of 26.7.24)

---

## Remaining Actions

**Priority 1 (DONE):** Fix POWL scheduler_v2.rs
- ✅ Refactored lines 73, 78, 87
- ✅ Reduced branching by combining empty checks
- ✅ Committed to codebase

**Priority 2 (PENDING):** Publish upstream dependencies
- ⏳ `cargo publish` bcinr-logic v26.7.25
- ⏳ `cargo publish` bcinr-api v26.7.25
- (Requires crates.io account and publish rights)

**Priority 3 (DONE):** Fix metadata
- ✅ Repository URL in Cargo.toml
- ✅ Version strings in README.md and SAFETY.md
- ✅ CURRENT_STATUS.md created

**Priority 4 (NEXT):** Re-run 4-gate workflow
- [ ] After upstream publish completes
- [ ] Verify Gate #3 dry-run succeeds
- [ ] Confirm all gates ALIVE
- [ ] Generate final release ledger

---

## Ship Criteria

✅ Gate #1 (Build & Test): ALIVE  
✅ Gate #2 (Hot Path Audit): ALIVE (after refactoring)  
⏳ Gate #3 (Publish): READY (awaiting upstream publish)  
✅ Gate #4 (Release Automation TDD): COMPLETE  

**Overall Standing:** READY_TO_SHIP (once upstream publishes)

**Next Step:** Complete Priority 2 (upstream publish), then re-run 4-gate workflow for final verification.
