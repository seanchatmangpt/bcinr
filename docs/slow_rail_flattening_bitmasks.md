# Flattening the Semantic Slow Rail into Hardware Bitmasks & Artifacts

The `bcinr` architecture strictly prohibits runtime allocations, graph traversals, and data-dependent loops on the hot path (the $CC=1$ Radon Law). To adhere to this, all high-level semantic graphs (RDF/SHACL) and temporal dependency plans (PDDL) in the slow rail are transformed into flat, deterministic bitmasks and fixed-size integer arrays before they reach the execution substrate.

## 1. RDF and SHACL Flattening into `Gamma_CMCA` Artifacts
The conversion of RDF graphs and SHACL shapes is handled by an offline generator (e.g., `crates/bcinr-cmca/quarantine/legacy-generator/generator.py`):

- **Offline Semantic Validation:** The generator parses the RDF structure (Turtle files) and runs a SHACL-equivalent verification pass in Python to validate ontological constraints (e.g., enforcing numeric types on `businessValue` and `Lens` exponents).
- **Off-Path Graph Resolution:** To avoid recursive tree traversal on the hot path, the generator statically computes the closure of transitive dependencies (`cmca:dependsOn`), flattening them into pre-calculated `downstreamConsequence` masses.
- **Q16.16 Arithmetic Transformation:** Floating-point metrics and multipliers are converted to saturating Q16.16 fixed-point numbers.
- **`PackedSemanticState` Artifacts:** The generator flattens all properties into an array of size $F$, outputting fixed-size Rust arrays of `PackedSemanticState` structs in a generated artifact (`Gamma_CMCA` contract). The hot path only indexes these structs and uses fixed-point math, entirely decoupled from the underlying RDF structure.

## 2. PDDL Temporal Plans Flattening into `pred_mask` & `succ_mask`
Execution scheduling constraints are transformed into $u64$ bitmasks by the POWL bridge (`crates/bcinr-pddl/src/powl_bridge.rs`):

- **Tape Bounding:** A sequential execution graph (`TemporalPlan`) is capped at 64 steps, mapping each step to a specific bit on a $u64$ integer to represent state.
- **Successor Mapping (`succ_mask`):** Each step gets a `succ_mask` with exactly one bit set corresponding to its index (`1u64 << i`), which is flipped in the global `completed_mask` when finished.
- **Precedence Bitmasks (`pred_mask`):** The temporal dependencies of each step are flattened into a `pred_mask`. The generator analyzes the timeline to find all steps that strictly precede the current one, then performs a transitive reduction to clear indirect dependencies (`direct_mask &= !ops[k].pred_mask`). 
- **Branchless Evaluation:** With the `pred_mask` populated, the execution engine (`bcinr-powl`) can determine if a step is ready using entirely constant-time bitwise logic: `(completed_mask & op.pred_mask) == op.pred_mask`.
