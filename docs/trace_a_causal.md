# DEEP TRACE A — CAUSAL: Semantic Edge Archaeology

## 1. Trace: Planner Result -> ActionOccurrence -> PddlCausalAnalyzer
The creation of causal edges begins when the classical search portfolio finds a sequentially ordered `Pddl8Tape`. 
In `crates/bcinr-pddl/src/mfw/planner.rs` (lines 398-402):
```rust
// --- Causal analysis ---
let occurrences = occurrences_from_tape(&tape, &epoch.actions);
let causal_plan = self
    .causal_analyzer
    .analyze(&epoch, &occurrences)
    .map_err(MfwPlanError::CausalAnalysis)?;
```
`occurrences_from_tape` converts the sequentially ordered operations into a vector of `ActionOccurrence`s, implicitly encoding execution time/timestamp into the list index. These are fed into `PddlCausalAnalyzer::analyze` (`crates/bcinr-pddl/src/causal.rs`).

## 2. Precedence Insertion Branch Inspection
Within `PddlCausalAnalyzer::analyze` (`crates/bcinr-pddl/src/causal.rs`, lines 138-162), there are two distinct passes over the $O(n^2)$ pairs. The first pass is the precedence insertion branch:
```rust
let mut precedes = StrictPartialOrder::default();
for i in 0..occurrences.len() {
    for j in (i + 1)..occurrences.len() {
        precedes.edges.insert(PrecedenceEdge {
            before: occurrences[i].id,
            after: occurrences[j].id,
        });
    }
}
```
**Conclusion:** Real paths do **NOT** derive order from causal support, delete/precondition interference, invariants, numeric flow, temporal law, or trajectory law. The order is derived **entirely from the list index (`i < j`)**, which corresponds to the classical sequential timestamp. 

While the actual causal mechanics (`analyze_pair` producing `DependenceWitness` via `CausalSupport`, `DeleteInterference`, and forward-simulated commutativity) are computed genuinely in the *second* loop, their findings are dumped into `independence.dependent` and `support_edges`. The canonical `precedes` order graph completely ignores this semantic information and just hardcodes a total execution order.

## 3. Minimum Surgery
To ensure index/timestamp never creates precedence, we must eliminate the unconditional first loop and build `precedes` lazily within the *second* loop, conditioned strictly on semantic dependence.

**Surgery:**
1. Delete the first `for i... for j...` loop.
2. In the second `for i... for j...` loop, check the result of `analyze_pair`.
3. If `witness` is `None` (meaning the pair is semantically `Dependent`), *then* insert `PrecedenceEdge { before: occurrences[i].id, after: occurrences[j].id }` into `precedes`.

## EDGE CARD: The Index-Driven Total Order

```yaml
EDGE CARD: PddlCausalAnalyzer_ArtificialTotalOrder
Status: BROKEN
Type: Precedence Edge Injection
Location: crates/bcinr-pddl/src/causal.rs:154-162
```
**Description:**
`PddlCausalAnalyzer` enforces a `StrictPartialOrder` that is functionally a *total order*. It iterates over all action occurrences and asserts that every `i` must precede every `j` (`i < j`), regardless of independence.

**Semantic Violation:**
A partial order in a causal plan should reflect necessary semantic ordering (causal support, delete/precondition interference). By inserting an edge for every `i < j`, the system collapses the concurrency complex back into a Von Neumann sequential sequence. Two non-interfering actions `A` and `B` will be strictly ordered `A -> B` purely because `A` was discovered before `B` in the BFS search rail.

**Impact:**
When `bcinr_powl::projection::PowlProjector` projects this `CausalPlan` into a `PowlModel`, it faithfully copies the broken `precedes` edges. This results in a POWL model devoid of structural concurrency—it projects a sequential line instead of a parallelized execution graph.
