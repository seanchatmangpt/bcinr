# DEEP TRACE B — K_E: Semantic Edge Archaeology

## 1. PddlConcurrencyAnalyzer Trace
The `PddlConcurrencyAnalyzer` implementation (`crates/bcinr-pddl/src/concurrency.rs`) only maps pairwise dependency relations into nonfaces. It strictly constructs 2-element nonfaces from the `CausalPlan::independence.dependent` set, at line 162:
```rust
let members = EventSet::empty().with(i).with(j);
```

## 2. The Capacity-Two Triple Nonface
**Location:** There is no code path in the real PDDL analyzer that constructs a capacity-two triple nonface. The exact lines constructing the `{A,B,C}` nonface are exclusively in hand-built test fixtures:
- `crates/bcinr-pddl/tests/mfw_capacity2_fixture.rs` (line 143): `let abc = EventSet::empty().with(0).with(1).with(2);`
- `crates/bcinr-mfw-ir/src/concurrency.rs` (line 134): `let abc = EventSet::empty().with(0).with(1).with(2);`

**Does source PDDL entail it?**
No. The real, PDDL-driven analyzer pair operates on classical STRIPS `Pddl8GroundAction` data. This structure possesses no numeric-fluent or capacity slots. It is structurally impossible to derive a "pairwise independent but jointly over capacity" conflict from the available PDDL data. The nonface is purely a synthetic mock.

## 3. Repair Decision
**Smallest Truthful Repair:** Return `Unsupported`.
The prompt explicitly states to never recommend pairwise approximation as exact. Because `PddlConcurrencyAnalyzer` only has access to boolean/classical STRIPS predicates and cannot access numeric-fluent data to derive genuine higher-order minimal nonfaces (such as a 3-way resource capacity conflict), its output is an approximation if capacity constraints exist. To maintain rigorous exactness without hallucinating constraints, the analyzer must return `Unsupported` when requested to build a complex for any domain that relies on numeric/capacity bounds, or when the data required for a full nonface representation is absent.

## 4. EDGE CARD

### EDGE CARD: Broken K_E Concurrency Lift (Pairwise Approximation)
- **Source Node:** `CausalPlan` (Pairwise Independence Relation)
- **Target Node:** `ExecutableConcurrencyComplex`
- **Edge Type:** `K_E` (Executable-concurrency complex derivation)
- **Status:** `BLOCKED` (Broken Edge)
- **Symptom:** `PddlConcurrencyAnalyzer` silently bounds its concurrency complex to 2-element nonfaces. True capacity or numeric-fluent conflicts (like a 3-way resource contention) are structurally undetectable.
- **Rule Violation:** Approximating a general simplicial complex using only pairwise (graph-level) cliques, without returning an exact result for capacity-constrained environments.
- **Resolution Path:** Return `Unsupported` when unable to construct exact higher-order nonfaces. Do not masquerade a pairwise approximation as a structurally sound executable concurrency complex.
