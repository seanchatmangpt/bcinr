# Structural Determinism Requirements (v26.7.15)
**Author:** `@turing_machine` (The Enforcer of Determinism)
**Domain:** `bcinr`, `praxis`, `mfact`

## 1. The Radon Law (CC=1) Across the V2 Bridge
The new V2 bridge (`CompiledPowlV2` -> `scheduler_tick_guarded_v2`) must strictly adhere to the Radon Law ($CC=1$).
- No `if`, `match`, or data-dependent `loop` structures are permitted in the hot-path execution loops.
- `scheduler_tick_guarded_v2` must evaluate readiness, apply concurrency guards, and mask the resulting execution via bitwise logic (`EventSet` intersections/unions) and branchless `PowlTapeLarge` traversals.
- The iteration over `v2::PowlTape` or `v2::PowlTapeLarge` must execute the exact same instruction stream regardless of the input data.

## 2. PddlConcurrencyAnalyzer & Capacity-Two Triples (Zero Bluffing)
The `PddlConcurrencyAnalyzer` currently relies on classical STRIPS data and artificially mocked test fixtures (the "hand-built Capacity-2 Complex") to simulate capacity constraints. This is a semantic bluff.
- **No LLM-Bluffs**: Any logic claiming to extract a capacity-two triple nonface from purely pairwise STRIPS data must be eradicated.
- **Exact Evaluation**: If numeric/capacity fluents are absent, the analyzer must return `Unsupported`. Do not approximate or fallback to pairwise nonfaces silently.
- **No Magic Constants**: `EventSet` masks must be derived dynamically from verifiable domain properties, never hardcoded (e.g., no `let abc = EventSet::empty().with(0).with(1).with(2);` in production logic).
- **No Artificial Branches**: Do not introduce conditional gates. Operations must structurally unify.

## 3. bcinr-cheat-scanner Constraints on Broken Edges

All five systematic cheat patterns are strictly prohibited in repairing the 5 broken semantic edges:
1. **Numeric Fluents in PDDL Analyzer**: Implement genuine numeric-fluent causal extraction without **Self-Canceling XOR** or **Circular Reference Oracles**.
2. **Witness-Backed Precedence**: `PddlCausalAnalyzer::analyze` must eliminate the artificial `i < j` loop. The precedence graph must strictly reflect dependence witnesses without **Artificial File-Length Inflation** or hidden heuristics.
3. **V2 Tape Scheduler Bridge**: Build native struct parsing without using **Magic Constants** to patch layout differences.
4. **Literal Ready-State in ExecutionReceipt**: Implement explicit `ready: EventSet` serialization in receipts without **Boilerplate Verification Claims** masking the `fired.is_subset_of(&ready)` subset check.
5. **Replay State-Space Dimensions**: Derive real metrics for `generalization` and `simplicity` instead of hardcoding Q16.16 zeros.

**Mandate**: Any refactoring must pass the `cargo make scan-cheats` gate. If the Substrate Integrity Score (SIS) falls below 100/100, the patch will be rejected.
