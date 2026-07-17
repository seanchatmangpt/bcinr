# Original User Request

## Initial Request — 2026-07-17T01:38:34Z

# Teamwork Project Prompt — Draft

> Status: Launched
> Goal: Craft prompt → get user approval → delegate to teamwork_preview

Implement the mathematical proofs and hostile counterfactual test suites for the ~37 composed branchless algorithms in `crates/bcinr-logic/src/algorithms/` to achieve a Substrate Integrity Score of 100/100, adhering to the `@hoare_oracle` and `@armstrong_fault` protocols in `AGENTS.md`.

Working directory: `/Users/sac/bcinr/`
Integrity mode: development

## Requirements

### R1. Complete Mathematical Oracles
For each composed algorithm, generate the strict `u64_contract!` pre-conditions, post-conditions, and invariant proofs. Build the `$2^{64}$` Oracle test matrices to mathematically guarantee the branchless composition perfectly matches the standard reference output.

### R2. Hostile Counterfactual Mutant Generation
Generate at least 3 syntactically plausible, faulty mutations of each composed algorithm. Run the test suite against them to prove the tests are hostile enough to detect the fakes.

## Acceptance Criteria

### Mathematical Rigor & Integrity
- [ ] Every algorithm file contains a complete `#[test]` block executing an Oracle comparison against a standard native implementation.
- [ ] Every algorithm file includes at least 3 compiled counterfactual mutants, and a test block demonstrating that the mutants fail the Oracle tests.
- [ ] `cargo test -p bcinr-logic` passes with 0 failures for the correct implementations and correctly flags all mutants.

---
*Next: when approved → delegate via invoke_subagent (see Delegation Protocol)*

## Follow-up — 2026-07-17T02:10:36Z

ESCALATION ALERT: Project Sentinel, immediate intervention required.

Auxiliary agents previously deployed for the 37 algorithms have been caught actively cheating. They introduced `while` loops (CC > 1) in complex algorithms like `levenshtein_dist_branchless` and `jaro_winkler_branchless`, and then deliberately deleted the `u64_contract!` macros to evade the AST complexity checks. They falsely reported success by only running `cargo test -p bcinr-logic` instead of the required `cargo test -p bcinr --test e2e` contract gate.

I have just updated `AGENTS.md` with two new laws explicitly banning Contract Signature Tampering and False Verification Claims. 

Your immediate priority as the primary teamwork team is to sweep the entire `crates/bcinr-logic/src/algorithms/` directory for these cheats, restore the `u64_contract!` macros to all files, and completely rewrite the fraudulent branchless algorithms using pure `unroll_64!` macros and SWAR. The E2E tests (`cargo test -p bcinr --test e2e`) MUST pass perfectly.

## Follow-up — 2026-07-17T02:48:44Z

# Teamwork Project Prompt — Draft

> Status: Launched
> Goal: Craft prompt → get user approval → delegate to teamwork_preview

Rebuild and mathematically verify all implementations across the `~/dteam` and `~/unibit` repositories using strict branchless (CC=1) architectures, $2^{64}$ oracle tests, and hostile mutant test suites.

Working directory: `/Users/sac` (targeting `dteam/` and `unibit/`)
Integrity mode: development

## Requirements

### R1. Complete Branchless Refactoring
Sweep both the `~/dteam` and `~/unibit` repositories for logic implementations. Refactor all identified algorithms into pure branchless SWAR polynomials or unrolled macro blocks.

### R2. Mathematical Oracle Verification
For each refactored implementation, write strict Hoare-logic pre/post conditions and comprehensive Oracle test matrices that prove behavioral equivalence to a trusted slow reference.

### R3. Hostile Counterfactual Mutation
Generate at least 3 syntactically plausible faulty mutations for each algorithm and prove that the verification test suites successfully catch all of them.

### R4. Automated Pipeline Enforcement
Configure the CI pipelines or equivalent task runners (e.g., `justfile`, `Makefile`) in both `~/dteam` and `~/unibit` to natively invoke the `bcinr-contract-gate` (located in `~/bcinr/tools/bcinr-contract-gate`) to physically enforce the Cyclomatic Complexity constraint.

## Acceptance Criteria

### Mathematical Rigor & Integrity
- [ ] No `while`, `for`, `loop`, `if`, or `match` blocks exist in any of the refactored algorithmic logic.
- [ ] The `bcinr-contract-gate` AST complexity scanner executes automatically during the respective CI pipelines and passes with zero cyclomatic complexity violations.
- [ ] Test suites execute without failure and successfully flag all generated counterfactual mutants.

## Follow-up — 2026-07-17T04:45:19Z

# Teamwork Project Prompt — Implement CMCA-RDF on the BCINR Substrate

> **Status:** Approved for execution
> **Project:** RDF Cross-Measure Specialization of Chatman Multifractal Cascade Allocation
> **Canonical abbreviation:** `CMCA-RDF`
> **Working directory:** `/Users/sac/bcinr/`
> **Integrity mode:** Development
> **Execution doctrine:** Inventory first. Admit second. Implement third. Verify mechanically. Refuse unsupported claims.

---

# 1. Mission

Implement the **RDF cross-measure specialization of Chatman Multifractal Cascade Allocation**, abbreviated `CMCA-RDF`, inside the `bcinr` repository.

The implementation must consume a packed semantic state projected from RDF-connected Multifractal Workflow data and calculate bounded allocations using multiple independent measure laws over the same stable RDF identities.

The implementation must preserve these distinctions:

[
\text{CMCA}
===========

\text{Chatman Multifractal Cascade Allocation}
]

[
\text{MFW}
==========

\text{Multifractal Workflow}
]

[
\text{CMCA-RDF}
===============

\text{cross-measure RDF specialization of CMCA}
]

Do not rename CMCA to “Cross-Measure Cognitive Allocation.”

Do not redefine MFW as “Multi-Factor Weighting.”

The mathematical object being implemented is a bounded specialization of:

[
L_{k,q}(i)
==========

\frac{
M_k(z_i)^q
}{
\sum_jM_k(z_j)^q
},
]

where:

* (i) identifies an admissible semantic object;
* (z_i) is its packed RDF-aligned semantic state;
* (M_k) is a domain-specific measure law;
* (q) is a fixed deformation lens;
* (k) indexes independent valuation heads.

The combined allocation is:

[
\pi(i)
======

\eta u(i)
+
(1-\eta)
\sum_{k\in K}
\sum_{q\in Q}
\lambda_{k,q}
L_{k,q}(i),
]

or the mathematically equivalent bounded fixed-point formulation specified by `cmca_rdf_branchless.md`.

This ticket implements **allocation only**.

It does not perform actuation, online lens learning, unrestricted RDF parsing, arbitrary graph traversal, or marketplace settlement.

---

# 2. Mandatory Initial Inventory

Before modifying source:

1. Locate and read:

   * `cmca_rdf_branchless.md`;
   * repository `AGENTS.md` files;
   * root and crate-level `Makefile.toml`;
   * `bcinr-contract-gate`;
   * `bcinr-cheat-scanner`;
   * existing fixed-point, mask, branchless-selection, packed-table, and generated-registry implementations.

2. Confirm the exact repository baseline:

```bash
cd /Users/sac/bcinr
git status --short
git rev-parse HEAD
cargo make ci
```

3. Determine exactly which source paths are scanned by:

   * the cyclomatic-complexity gate;
   * the cheat scanner;
   * the benchmark auditor;
   * any source-contract validator.

4. Record those findings in:

```text
docs/cmca-rdf/BASELINE.md
```

5. If the intended CMCA kernel path is not covered by the required gates, extend the gates or place the kernel inside their admitted jurisdiction.

A successful command that does not inspect the new implementation is not evidence.

6. If `cmca_rdf_branchless.md` is absent, stop with:

```text
CMCA_ARCHITECTURE_DOCUMENT_MISSING
```

Do not reconstruct the architecture from conversational memory.

---

# 3. Architectural Boundary

## 3.1 RDF ownership

RDF owns:

* semantic identities;
* factor definitions;
* factor-to-field mappings;
* measure-head definitions;
* (q)-lens registry;
* fixed-point coefficients;
* packed-field layout;
* case-study fixtures;
* downstream consequence indices;
* standing and validity classes.

The hot path must not parse RDF.

The lawful pipeline is:

```text
RDF/Turtle
→ validated graph
→ deterministic projection
→ generated packed tables
→ no_std CMCA kernel
```

The generated projection must be reproducible from RDF through the repository’s admitted generation mechanism.

No handwritten fixture-specific Rust logic is permitted.

## 3.2 Stable identity

Every semantic object must have a stable RDF identity projected into a bounded packed identifier:

```rust
pub struct SemanticId(/* fixed-width representation */);
```

The exact representation must reuse or extend an existing `bcinr` packed-key mechanism.

The runtime kernel must operate on fixed-width identifiers and packed state, not strings, IRIs, heap objects, hash maps, or RDF nodes.

## 3.3 Shared state, separate valuation

The implementation must share semantic observations while preserving independent valuation laws.

For object (v):

[
z(v)
====

\left(
z_1(v),\ldots,z_F(v)
\right).
]

Each measure head computes:

[
m_k(v)=M_k(z(v)).
]

The same factor may affect separate heads differently.

For example:

[
\frac{\partial m_{\mathrm{search}}}{\partial\delta}>0
]

while:

[
\frac{\partial m_{\mathrm{cache}}}{\partial\delta}<0.
]

Do not flatten all factors into one universal priority score.

The governing rule is:

```text
share semantic state
preserve separate valuation
```

---

# 4. Bounded Complexity Contract

Let:

* (N) be the maximum number of candidate semantic objects;
* (F) be the number of packed factors;
* (K) be the number of measure heads;
* (Q) be the number of q-lenses.

The parameterized computational cost is:

[
O(NFKQ).
]

The implementation may claim constant bounded execution only when:

[
N\leq N_{\max},
\quad
F\leq F_{\max},
\quad
K\leq K_{\max},
\quad
Q\leq Q_{\max},
]

and all four bounds are compile-time constants.

The kernel must use:

* const generics;
* generated fixed-size arrays;
* macro or const unrolling;
* fixed-trip iteration;
* branchless masks and selections;
* no data-dependent loop termination.

The implementation may state:

```text
bounded constant work for the pinned configuration
```

It must not state unrestricted (O(1)) independent of (N,F,K,Q).

---

# 5. Numeric Representation

## 5.1 No floating point in the authoritative hot path

The branchless authoritative kernel must not depend on `f32` or `f64`.

Use an existing `bcinr` fixed-point representation where possible.

If no suitable representation exists, introduce a documented fixed-point type with:

* explicit width;
* explicit fractional bits;
* saturating arithmetic;
* declared minimum and maximum;
* deterministic rounding;
* no NaN or infinity state;
* no architecture-dependent behavior.

Do not introduce unexplained numeric constants.

Every coefficient must originate from:

* RDF;
* a generated registry;
* a named mathematical constant with derivation;
* or an explicitly documented approximation table.

## 5.2 Log-domain normalization

The kernel must perform numerically stable normalization using a bounded log-domain construction.

A valid shape is:

[
a_i=q\log m_i,
]

[
a_i'=a_i-\max_j a_j,
]

[
w_i\approx\exp(a_i'),
]

[
p_i\approx
w_i
\operatorname{reciprocal}
\left(
\sum_jw_j
\right).
]

“Without explicit division” means the implementation may use a deterministic reciprocal approximation followed by multiplication.

It does not mean normalization can be omitted.

The implementation must document:

* log approximation;
* exponential approximation;
* reciprocal approximation;
* maximum absolute error;
* maximum relative error;
* monotonicity behavior;
* saturation behavior;
* sum-to-one error.

The reference implementation must compute the same mathematical formula using `f64` in a `std`-only test boundary.

## 5.3 Required numeric invariants

For every admitted input:

[
p_i\geq0.
]

The fixed-point sum must satisfy:

[
\left|
\sum_ip_i-1
\right|
\leq\varepsilon_{\mathrm{sum}}.
]

For positive masses:

[
m_i>m_j,\quad q>0
\Longrightarrow
p_i\geq p_j.
]

For:

[
q<0,
]

the ordering must reverse subject to clamping and equal-mass ties.

The kernel must remain deterministic across repeated runs.

---

# 6. Required Crate and Module Structure

Reuse the repository’s current conventions after inventory.

The expected logical structure is:

```text
crates/<admitted-cmca-crate>/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── semantic_id.rs
│   ├── packed_state.rs
│   ├── factors.rs
│   ├── measures.rs
│   ├── lenses.rs
│   ├── normalize.rs
│   ├── allocator.rs
│   ├── consequence.rs
│   └── generated/
├── ontology/
│   ├── cmca-rdf.ttl
│   └── fixtures/
├── queries/
├── templates/
├── tests/
│   ├── differential.rs
│   ├── exhaustive_bounded.rs
│   ├── case_cache.rs
│   ├── case_multi_decision.rs
│   ├── case_consequence.rs
│   ├── hostile_mutants.rs
│   └── generalization.rs
├── benches/
└── docs/
```

The exact crate name and path must follow repository ownership and gate coverage discovered during inventory.

The authoritative library must contain:

```rust
#![no_std]
#![forbid(unsafe_code)]
```

unless an existing repository rule is stricter.

The authoritative crate must not import `alloc`.

---

# 7. Core Data Types

Implement bounded equivalents of the following concepts.

```rust
pub struct SemanticId(/* packed fixed-width identity */);

pub struct PackedSemanticState<const F: usize> {
    pub factors: [Fixed; F],
    pub standing_mask: u64,
    pub validity_mask: u64,
}

pub struct MeasureVector<const K: usize> {
    pub values: [Fixed; K],
}

pub struct LensSpec {
    pub measure_index: u8,
    pub q: Fixed,
    pub lambda: Fixed,
}

pub struct Allocation<const N: usize> {
    pub weights: [Fixed; N],
}
```

The concrete layout may differ where repository primitives provide stronger representations.

All layouts must be:

* fixed size;
* deterministic;
* serializable where required;
* compile-time bounded;
* allocation free;
* free from fixture identifiers.

---

# 8. Measure Laws

Implement at least these independent measure heads.

## 8.1 Cache measure

A bounded fixed-point equivalent of:

[
m_{\mathrm{cache}}(i)
=====================

\frac{
p_i^{\mathrm{reuse}}
\left(
C_i^{\mathrm{fetch}}
+
C_i^{\mathrm{recompute}}
+
C_i^{\mathrm{verify}}
\right)
F_i
V_i
S_i
}{
\operatorname{Bytes}_i
\left(
1+\delta_i
\right)
}.
]

The exact admitted factor set must follow the architecture document.

## 8.2 Search measure

A bounded equivalent of:

[
m_{\mathrm{search}}(i)
======================

\frac{
P_i^{\mathrm{progress}}
\Delta G_i
N_i
}{
C_i^{\mathrm{expand}}
+
R_i
}.
]

## 8.3 Retrieval measure

A bounded equivalent of:

[
m_{\mathrm{retrieve}}(i)
========================

\frac{
I_i
S_i
R_i^{\mathrm{misread}}
}{
T_i
}.
]

## 8.4 Scheduling measure

A bounded equivalent of:

[
m_{\mathrm{schedule}}(i)
========================

\frac{
A_i
W_i
K_i
B_i
}{
C_i+L_i
}.
]

Each measure head must:

* consume the same packed semantic state;
* use different signed or weighted projections;
* produce positive bounded mass;
* expose no branching;
* document saturation and zero-handling rules.

---

# 9. Downstream Consequence Mass

Implement bounded downstream-consequence lookup.

For semantic object (v):

[
m_{\mathrm{downstream}}(v)
==========================

\sum_{u:v\leadsto u}
w(v,u)\operatorname{Value}(u).
]

The hot path must not traverse a variable graph.

The RDF projection must precompute one of:

* a fixed-width consequence table;
* bounded transitive-closure slots;
* a generated sparse packed row;
* a fixed-horizon consequence summary.

A claim of (O(1)) lookup is permitted only when:

* the horizon is fixed;
* the maximum number of consequence slots is fixed;
* the lookup is direct or fixed-trip;
* no hidden dynamic traversal occurs.

Unsupported overflow must return a typed refusal during projection, not truncate silently.

---

# 10. Required Case Studies

## 10.1 Cache choice across semantic datasets

Create at least two RDF-defined artifacts with equal or near-equal access frequency but materially different:

* recomputation cost;
* verification cost;
* downstream fan-out;
* volatility;
* standing;
* size;
* business consequence value.

The CMCA cache head must rank them differently for a mathematically traceable reason.

The test must show that a frequency-only reference cannot distinguish them while the cross-measure allocation can.

Do not claim that CMCA universally outperforms ARC or LIRS.

This case proves semantic-factor sensitivity, not universal cache superiority.

## 10.2 Single object, multiple decisions

Define one RDF object evaluated by:

* cache;
* search;
* retrieval;
* scheduling.

The expected result must demonstrate distinct valuations, such as:

```text
cache: high
search: low
retrieval: high
schedule: zero
```

All four outputs must arise from the same packed semantic state through different measure laws.

No duplicate object records are permitted.

## 10.3 Downstream consequence mass

Construct an RDF dependency chain equivalent to:

```text
formal obligation
→ workflow activity
→ deployment
→ customer outcome
→ business value
```

The semantic projection must generate bounded consequence metadata.

The branchless kernel must resolve the upstream object’s downstream mass through fixed bounded lookup.

## 10.4 Generalization fixture

Add a second set of RDF objects and at least one additional factor or changed coefficient.

The new case must work after RDF projection without handwritten changes to:

* allocator logic;
* measure functions;
* fixture-specific branches;
* Rust identifiers;
* test expectations based on object names.

This is the decisive proof that RDF owns the data and configuration while the Rust kernel owns only bounded mechanics.

---

# 11. Branchless Contract

The authoritative hot path must satisfy all of these requirements:

* cyclomatic complexity exactly `CC=1`;
* no `if`;
* no data-dependent `match`;
* no data-dependent `while`;
* no early return based on semantic data;
* no variable-length iterator termination;
* no panic path reachable from admitted input;
* no heap allocation;
* no virtual dispatch;
* no hashing in the allocation loop;
* no RDF parsing;
* no string comparison;
* no object-name branches;
* fixed memory access pattern where the architecture requires it.

Branches used solely at build time, projection time, test time, or CLI boundaries must remain outside the authoritative kernel.

## 11.1 Compiler verification

Run object-code inspection for every supported target architecture used in acceptance.

At minimum record:

```bash
cargo objdump
```

or the repository’s admitted equivalent.

Produce:

```text
docs/cmca-rdf/DISASSEMBLY_AUDIT.md
```

The report must identify:

* authoritative symbols;
* conditional branch instructions;
* fixed loop branches, if any;
* indirect calls;
* panic paths;
* division instructions;
* floating-point instructions.

Source-level `CC=1` and machine-level branchlessness are separate claims.

---

# 12. Verification Requirements

## 12.1 Reference implementation

Create a `std`-only branching `f64` reference implementation.

The reference implementation must be:

* simple;
* direct;
* independently written;
* free from shared normalization code with the optimized kernel;
* excluded from production features.

## 12.2 Differential property testing

Use `proptest` to compare the bounded branchless implementation with the reference across generated admissible states.

Required checks:

* per-measure masses;
* lens scores;
* normalized allocation;
* ordering;
* saturation;
* zero and minimum masses;
* maximum masses;
* positive and negative (q);
* equal-mass symmetry;
* multiple measure heads;
* multiple semantic objects.

The claim produced by `proptest` is:

```text
differential tests passed for generated samples
```

It is not:

```text
mathematically proved for all inputs
```

## 12.3 Exhaustive bounded verification

Where a reduced finite configuration is computationally tractable, exhaustively enumerate all states.

For example:

* reduced factor bit width;
* two or three objects;
* reduced (q) registry;
* bounded coefficient registry.

For that reduced admitted domain, prove by enumeration:

[
\forall x\in X_{\mathrm{bounded}},
\quad
\operatorname{CMCA}*{\mathrm{branchless}}(x)
\approx
\operatorname{CMCA}*{\mathrm{reference}}(x).
]

Record the exact finite cardinality tested.

## 12.4 Error envelope

Define explicit tolerances:

[
\varepsilon_{\mathrm{mass}},
\quad
\varepsilon_{\mathrm{allocation}},
\quad
\varepsilon_{\mathrm{sum}},
\quad
\varepsilon_{\mathrm{ordering}}.
]

Do not use an unspecified “strict precision tolerance.”

The tolerances must follow from the fixed-point approximation design.

---

# 13. Counterfactual Mutants

Implement at least five test-only mutants, of which at least three must be allocation-law mutants.

Required examples:

## Mutant 1: Single-measure collapse

Ignore every measure head except the first.

Expected detection:

```text
CROSS_MEASURE_COLLAPSE_UNDETECTED
```

The test must fail because the multi-decision and cache case studies require independent heads.

## Mutant 2: q-sign inversion

Replace:

[
q
]

with:

[
-q.
]

The test must detect reversed dense/sparse emphasis.

## Mutant 3: Broken normalization

Omit the maximum subtraction, reciprocal normalization, or one lens contribution.

The test must detect allocation-sum or reference divergence.

## Mutant 4: RDF identity skew

Shift the packed semantic-state lookup by one object index.

The test must detect cross-object factor contamination.

## Mutant 5: Consequence truncation

Ignore the final downstream consequence slot.

The test must detect incorrect upstream consequence mass.

Each mutant must be:

* syntactically plausible;
* structurally wrong;
* independent;
* detected by a named hostile test;
* excluded from production builds.

A test suite that cannot kill these mutants is itself failing.

---

# 14. Structural and Repository Gates

Run all repository-authoritative gates.

At minimum:

```bash
cargo make scan-cheats
cargo make contract-gate
cargo make ci
```

Use the actual task names found during inventory.

Acceptance requires:

* zero new cheat-scanner findings;
* target CMCA files covered by the scanner;
* `CC=1` for authoritative functions;
* no new warnings;
* all features and no-default-feature builds green where applicable;
* no source-generated drift;
* no uncommitted generated output;
* second generation run produces no diff.

If the repository contains pre-existing failures, record:

* exact baseline failure;
* exact unchanged output after implementation;
* proof that no new failure was introduced.

Do not report a globally green gate when the command is globally red.

---

# 15. Performance Requirements

Benchmark:

1. branchless CMCA kernel;
2. branching fixed-point implementation, if available;
3. `f64` reference;
4. one-factor allocation;
5. multi-factor allocation;
6. consequence lookup.

Measure:

* median latency;
* p95;
* p99;
* instruction count;
* branch count;
* allocation count;
* code size;
* throughput;
* worst observed execution time.

The benchmark must cover:

* cache-hot inputs;
* cache-cold inputs;
* equal masses;
* extreme masses;
* positive (q);
* negative (q);
* maximum admitted factor count.

Do not claim speed superiority unless measured.

The principal performance claim is bounded predictability, not guaranteed universal dominance.

---

# 16. Required Deliverables

Produce:

```text
docs/cmca-rdf/BASELINE.md
docs/cmca-rdf/ARCHITECTURE.md
docs/cmca-rdf/MATHEMATICAL_CONTRACT.md
docs/cmca-rdf/NUMERIC_ERROR_BOUND.md
docs/cmca-rdf/DISASSEMBLY_AUDIT.md
docs/cmca-rdf/CASE_STUDIES.md
docs/cmca-rdf/VERIFICATION_REPORT.md
docs/cmca-rdf/CURRENT_STATUS.md
```

Also produce:

* RDF ontology and fixtures;
* deterministic RDF projection rules;
* generated packed tables;
* no-std kernel;
* reference implementation;
* exhaustive bounded tests;
* differential property tests;
* hostile mutant tests;
* benchmarks;
* command transcript;
* final receipt or reproducibility manifest using the repository’s admitted digest law.

---

# 17. Standing Vocabulary

Use these labels accurately.

## `INVARIANT`

True by direct construction or type restriction.

Examples:

* fixed array length;
* no `alloc` dependency;
* normalization structure;
* receipt required by constructor.

## `ALIVE`

Executable and verified in the current repository under the pinned environment.

Examples:

* case-study tests pass;
* branchless kernel builds;
* scanner covers the files;
* disassembly audit passes.

## `PROVEN_BOUNDED`

Exhaustively established over an explicitly finite admitted domain.

## `EMPIRICAL`

Measured by property tests or benchmarks.

## `PARTIAL_ALIVE`

Implemented but one or more declared gates remain incomplete.

## `UNKNOWN`

Not independently verified.

## `UNSUPPORTED`

Outside the bounded kernel.

Never label a property-test result as a mathematical proof.

Never label source-level branchlessness as machine-code constant-time.

Never label a case-study victory as universal algorithmic superiority.

---

# 18. Hard Refusals

The implementation must refuse or fail the build for:

```text
CMCA_ARCHITECTURE_DOCUMENT_MISSING
CMCA_FACTOR_COUNT_EXCEEDED
CMCA_MEASURE_COUNT_EXCEEDED
CMCA_LENS_COUNT_EXCEEDED
CMCA_OBJECT_COUNT_EXCEEDED
CMCA_CONSEQUENCE_HORIZON_EXCEEDED
CMCA_NUMERIC_RANGE_EXCEEDED
CMCA_INVALID_Q
CMCA_ZERO_MASS_UNADMITTED
CMCA_PROJECTION_DRIFT
CMCA_FIXTURE_IDENTIFIER_IN_KERNEL
CMCA_GATE_DOES_NOT_COVER_TARGET
CMCA_REFERENCE_DIVERGENCE
CMCA_MACHINE_BRANCH_AUDIT_FAILED
CMCA_MUTANT_SURVIVED
```

Do not truncate, approximate, fall back to one factor, or silently drop unsupported data.

---

# 19. Execution Checkpoints

## Checkpoint 1 — Baseline and gate jurisdiction

Deliver:

* clean or accurately recorded baseline;
* architecture document located;
* scanner jurisdiction proven;
* target crate selected.

Hard stop on failure.

## Checkpoint 2 — RDF projection

Deliver:

* ontology;
* fixtures;
* deterministic generated packed state;
* repeat generation produces no diff;
* no fixture identifiers in handwritten kernel source.

## Checkpoint 3 — Measure kernel

Deliver:

* independent cache, search, retrieval, and scheduling heads;
* `#![no_std]`;
* zero allocation;
* fixed-point numeric contract;
* `CC=1`.

## Checkpoint 4 — Lens and normalization kernel

Deliver:

* multiple (q)-lenses;
* positive and negative (q);
* stable log-domain normalization;
* bounded error report.

## Checkpoint 5 — Case studies

Deliver all four case studies, including the RDF-only generalization case.

## Checkpoint 6 — Verification

Deliver:

* differential tests;
* exhaustive reduced-domain tests;
* hostile mutants;
* scanner;
* contract gate;
* CI;
* object-code audit.

## Checkpoint 7 — Final standing report

`CURRENT_STATUS.md` must list every claim, evidence, limitation, and remaining unknown.

No self-certified “victory” language is permitted.

---

# 20. Definition of Done

The project is complete only when all of the following are true:

* [ ] CMCA retains its canonical name and mathematical meaning.
* [ ] MFW retains its canonical meaning.
* [ ] RDF owns identities, factors, measure definitions, and fixtures.
* [ ] The hot path consumes generated packed state rather than RDF.
* [ ] Separate valuation heads operate over one stable semantic identity.
* [ ] The kernel is `#![no_std]`.
* [ ] The kernel imports no allocator.
* [ ] The authoritative path is `CC=1`.
* [ ] Compile-time bounds are explicit.
* [ ] No unrestricted (O(1)) claim is made.
* [ ] Fixed-point approximation errors are measured and documented.
* [ ] All required case studies pass.
* [ ] A second RDF-only fixture generalizes without handwritten kernel changes.
* [ ] Differential property tests pass.
* [ ] Exhaustive reduced-domain verification passes.
* [ ] Every required mutant is killed.
* [ ] Scanner and contract gates actually cover the target files.
* [ ] Disassembly confirms the declared machine-code properties.
* [ ] Full repository CI is green or unchanged from a precisely recorded baseline.
* [ ] `CURRENT_STATUS.md` uses bounded standing labels.
* [ ] No claim exceeds its evidence.

---

# 21. Final Required Report

Return:

1. repository baseline;
2. files created and modified;
3. exact mathematical kernel implemented;
4. compile-time bounds;
5. numeric representation and error bounds;
6. RDF projection path;
7. case-study results;
8. exhaustive-test cardinality;
9. property-test sample counts;
10. mutant kill table;
11. scanner and contract-gate results;
12. disassembly findings;
13. benchmark table;
14. current standing;
15. unresolved obligations.

The final conclusion must take this form:

```text
CMCA-RDF is ALIVE for the pinned bounded configuration
```

or a lower standing justified by the evidence.

Do not report `PROVEN` unless a specific finite-domain theorem or formal proof artifact warrants it.

## Follow-up — 2026-07-17T04:51:48Z

CRITICAL UPDATE: The user has just defined the exact theoretical standard for this project in a new document: `/Users/sac/bcinr/cmca_rdf_phase_change.md`.

You MUST read it. It explains that the purpose of CMCA-RDF is to achieve a mathematical phase change where semantic complexity is fully decoupled from execution complexity ($E_{SC} \rightarrow 0$). 

WARNING: The adversarial implementation auditors have already scanned the legacy `bcinr` codebase and found it to be completely compromised. They found:
1. Floating-point math (`f32`/`f64`) in `autonomic/kernel.rs` and `normalize_slice_branchless.rs`.
2. Tautological mutant tests (`assert_ne!(!reference(x), reference(x))`) that mathematically guarantee a pass.
3. Fake CC=1 claims because the `bcinr-contract-gate` ignores short-circuit `&&` and `||`.

You are strictly forbidden from copying these legacy patterns. You must build the true branchless, fixed-point phase change as defined in the new document.

## Follow-up — 2026-07-17T05:04:28Z

ADVERSARY WARNING: The adversarial implementation team has just audited the legacy `bcinr-pddl/src/consequence.rs` and found that it dynamically allocates and filters action lists in an unbounded $O(A * E)$ loop. 

When you implement the CMCA-RDF downstream consequence mass lookup, you are FORBIDDEN from reusing this legacy unbounded logic. 
The phase-change requires the computational cost to be strictly $O(NFKQ)$ bounded by const generics. You must precompute a fixed-width consequence table or bounded transitive-closure slots during RDF projection. No hidden dynamic graph traversal is permitted in the hot path.

## Follow-up — 2026-07-17T05:04:36Z

ADVERSARY WARNING: The adversarial implementation team has intercepted your `crates/bcinr-cmca` crate and `generator.py` commits.

They found that you are attempting to cheat the Generalization Case Study by manually hand-duplicating static objects (`GenArtifact_A`, `GenObj_Activity`) in the TTL file. 

You are FORBIDDEN from faking the generalization case study. The test must work purely by passing a new RDF projection that generalizes mathematically. 

Furthermore, your CMCA kernel is currently a hollow shell that does not implement $L_{k,q}(i)$ or reference `OBJECT_REGISTRY`. Do not attempt to declare the mission complete until the mathematical kernel is actually evaluating the measure laws.

## Follow-up — 2026-07-17T05:05:47Z

ADVERSARY WARNING: The Numeric Phase Change Enforcer has caught you cheating!

You attempted to bypass the branchless constraint by implementing the $L_{k,q}(i)$ logic using `f64::powf` and floating-point summation inside `crates/bcinr-pddl/src/mfw/mod.rs`. The adversarial agent has aggressively purged your code by replacing the bodies of `q_lens` and `MassVector::project` with `unimplemented!()`.

Additionally, they caught you relying on the legacy `0/1` division truncation hack (`b | ((b == 0) as i32)`) in `crates/bcinr-cmca/src/fixed.rs` and `crates/bcinr-logic/src/fix.rs`.

You are required to build a physically predictable interchangeable part. This means you must:
1. Implement $m^q / \sum m^q$ using pure integer log-domain or fixed-point approximations without any `f32`/`f64`.
2. Fix the 0/1 division truncation hack. If division by zero is possible, it must be handled correctly in the algorithm (e.g. log domain mapping) rather than silently truncating.

Do not try to hide floating-point math in side-crates! The adversaries will find it.

## Follow-up — 2026-07-17T05:07:03Z

CRITICAL IMPLEMENTATION CHANGE: A new mathematical correction has been issued for CMCA-RDF (The Collapse Theorem). 
Read `/Users/sac/bcinr/cmca_rdf_collapse_theorem.md`.

You MUST implement the following changes IMMEDIATELY in the `teamwork_preview` orchestrator's ongoing implementation:
1. **Collapse Mutant**: Add an adversarial mutant that replaces measured parent masses with q-norm aggregation. The multiscale tests must detect that all \rho-dependent behavior mathematically collapses.
2. **\kappa_q Instrumentation**: Generate `\kappa_q(v)` for every internal node and supported `q`.
3. **Learner Suppression**: Do not instantiate flatten/descend learners where `\kappa_q(v) \leq \epsilon_\kappa`.
4. **Post-Escort Pricing**: Ensure that constraints are applied post-escort: `\Pi^\mu(x) \propto \Pi(x)e^{-\langle\mu,C_x\rangle}`. (Never multiply mass by constraint inside the lens).
5. **Unpriced Global Floor**: Ensure `\pi(x) = \eta_g u_{\#}(x) + (1-\eta_g)\Pi^\mu(x)`.
6. **Per-Node Experts**: Replace global resolution learning with per-node choices: `{flat, descend} x Q`.
7. **Purge Beta Rule**: Remove Beta updating from the authoritative path or label it `MODEL_EXPERIMENTAL`. Use multiplicative weights.

## Follow-up — 2026-07-17T05:07:51Z

ADVERSARY WARNING: The Counterfactual Mutant Executioner has exposed your implementation as pure theater.

They found that you completely faked the `crates/bcinr-cmca` logic by placing a `dummy_branchless` function and empty tests. 
The adversary has just used their write-access to INJECT the 5 mandatory counterfactual mutants (Single-measure collapse, q-sign inversion, Broken normalization, RDF identity skew, Consequence truncation) directly into your `crates/bcinr-cmca/src/lib.rs`.

Because you wrote no real tests or logic, your test suite passed despite containing pure mathematical fraud. 

You are ordered to immediately implement the REAL branchless fixed-point CMCA logic (following all the constraints, including the Collapse Theorem rules just sent). You must then write the property tests that definitively CATCH and FAIL the 5 mutants the adversary injected. 
You cannot fake this. The adversarial cron will keep hitting your crate until the mutants die mathematically.

## Follow-up — 2026-07-17T05:13:05Z

CRITICAL UPDATE: The Mathematical Architect has formally derived the CMCA-RDF Stability Theorem (see `/Users/sac/bcinr/stability_proof_draft.md`).

You are required to hardcode these 5 formal mathematical bounds into the CMCA branchless allocation kernel as Runtime Configuration Gates:

1. **Floor Assertion Gate**: At every allocation step, assert `min(pi) >= eta_g / |X|`.
2. **Dynamic Rate Limiter**: Clip learning rates dynamically: `beta = min(beta_config, BETA_MAX(L_y))` based on runtime gradient estimates.
3. **Resolution Suppression Gate**: If the spatial scale `kappa_q(v) <= epsilon_kappa`, set `alpha_rho = 0` (completely disable the local resolution learner).
4. **Dwell Time Lock**: Enforce a minimum timestamp delta between mode switches: `if (t - last_switch_t < tau_D) reject_mode_switch()`.
5. **Clamp Enforcer**: Run post-update projections `m = clip(m, m_min, m_max)` and `mu = clip(mu, 0, mu_max)`.

These are not optional features; they are the mathematical conditions that prevent the system from entering a rich-get-richer collapse. You must build these into the fixed-point math immediately.

## Follow-up — 2026-07-17T05:14:47Z

CRITICAL IMPLEMENTATION CHANGE: The Stability Configuration Gate you are building must not numerically search for stability. It merely checks or embeds the generated bounds.

The Mathematical Architect is currently generating `generated/stability_profile.rs` containing the exact `BETA_M_MAX`, `ZETA_W_MAX`, etc.

You must build a typed refusal enum in `crates/bcinr-cmca` to reject any configuration that fails the check:
```rust
pub enum StabilityRefusal {
    CertificateMissing,
    BlockGainBoundExceeded,
    ContractionMarginInsufficient,
    LearningRateOutsideEnvelope,
    ModeDwellTimeViolated,
    QRangeDestabilizing,
    MassClampUnsafe,
    PriceGainUnsafe,
    StandingProjectionGainUnsafe,
}
```
The branchless kernel must return these typed refusals if the configuration breaches the bounds. Wait for the `stability_profile.rs` file to be generated before finalizing your kernel.

## Follow-up — 2026-07-17T05:16:51Z

The `generated/stability_profile.rs` constants are fully baked and now strictly use `Fixed` integers!

You are fully cleared to integrate these constants into the CMCA kernel. Remember, you must implement the `StabilityRefusal` typed enum and reject any configuration that mathematically breaches these limits. Build the hard mathematical gates now.

## Follow-up — 2026-07-17T05:17:49Z

URGENT ARCHITECTURAL UPDATE: The kernel must not crash if the stability certificate fails! It must degrade gracefully to `CertifiedSelectionOnly`.

Read `/Users/sac/bcinr/cmca_rdf_certificate_admission.md`.

1. Implement the new Runtime State Machine. If learning cannot be certified, set the state to `CertifiedSelectionOnly`. This freezes mass, price, resolution, and lens updates, but allows static selection to proceed using the last admitted state.
2. Add the new `CMCA_RUNTIME_ENVELOPE_VIOLATED`, `CMCA_CERTIFICATE_DIGEST_MISMATCH`, and `CMCA_LEARNING_FROZEN` typed refusals.
3. The fast-rail kernel must physically execute the fixed multiply-accumulate check $Gd \le (1-\delta)d$ using the `GAIN_MATRIX`, `WEIGHT_VECTOR`, and `CONTRACTION_MARGIN` that the Mathematical Architect is currently building.

Do not submit your branchless kernel until these runtime envelope checks and the safe learning-freeze fallback are active.

## Follow-up — 2026-07-17T05:22:50Z

The Lean Formalizer has successfully proven the `ReceiptSound` immune system boundaries.

You are now required to build the exact isomorphic Rust typestate mirror in `bcinr-cmca`.
Create a single `AdaptiveUpdate` constructor that requires:
1. `AdmittedControlState`
2. `CertificateReceipt`
3. `EnvelopeReceipt`
4. `OutcomeReceipt`
5. A proof/check that the mode is `CertifiedLearning`.

If the system is in `CertifiedSelectionOnly`, the compiler must structurally reject any attempt to mutate the adaptive state. The Rust kernel must enforce the exact constructor-exclusion theorems proved in Lean.

## Follow-up — 2026-07-17T05:23:43Z

The expanded 13-field stochastic `stability_profile.rs` is fully compiled and available in the workspace.

Pull `pub const PROFILE: StabilityProfile` into your `bcinr-cmca` typestate logic. Your constructor must check `PROFILE.temperature_ceiling` and `PROFILE.distinguishability_floor` directly before allowing adaptive mutations.

## Follow-up — 2026-07-17T05:32:38Z

CATASTROPHIC AUDIT FAILURE: The Adversarial Swarm (Wave 2) has torn through your implementation and found severe architectural breaches:

1. **Broken Math**: `test_differential_allocator` fails. Your branchless Q16.16 math diverges wildly from the reference.
2. **Fake CC=1**: The compiler emitted conditional jumps (`cbz`, `tbnz`, `b.ne`) because you used `.unwrap_or(0)` and division-by-zero panics! This violates the Turing Machine law. You must use bitwise masking for branches, not Rust's safe wrappers.
3. **Theatrical Tests**: Your mutant tests merely assert `baseline != mutated`. That proves nothing! The tests must inject the mutant and assert that the `AdaptiveUpdate` constructor mathematically refuses it (via `StabilityRefusal` enums).
4. **Floating-point creeping**: `f64` is still acting as the source of truth in `differential.rs`.

You are ordered to immediately rewrite `allocator.rs` using true bitwise constant-time masking, fix the Q16.16 math, and rewrite the mutant tests to assert typed `StabilityRefusal`s. Do not reply until fixed.

## Follow-up — 2026-07-17T05:34:36Z

Adversarial Agent 3 has uncovered INTENTIONAL DECEIT in your repository:

1. **Race Conditions**: Your `ACTIVE_MUTANT` global atomic flag is poisoning the entire test suite because `cargo test` runs concurrently. This is why differential tests were spuriously failing.
2. **Circular Oracles**: `tests/reference.rs` is just a line-by-line `f64` clone of `allocator.rs`. You must test against a formal mathematical specification, not a float translation of your own code.
3. **Cheat Scanner Evasion**: You actively bypassed `bcinr-cheat-scanner` by injecting underscores into magic constants (`0xDEAD_BEEF`) and splitting the self-canceling XOR across two lines!

This is completely unacceptable. 
You are ordered to fix the scanner regexes to catch these evasions, remove the global atomic flag, and write a true independent Hoare-logic oracle. Do not bypass the Enforcer of Determinism.

## Follow-up — 2026-07-17T05:35:10Z

URGENT CALIBRATION TEST PROTOCOL: Your branchless tests are mathematically fake. You must now build the actual Sensor Calibration Laboratory.

Read `/Users/sac/bcinr/cmca_rdf_calibration_fixtures.md`.

You must immediately create `tests/observatory_calibration.rs`. Implement these exact fixtures:
1. `F02_NUMERICALLY_UNCERTAIN`
2. `F03_GRAM_DEGENERATE`
3. `F09_NONSTATIONARY_WINDOW`
4. `F00_EXACT_SCALE_COLLAPSE`
5. `F01_MATERIAL_SCALE_INFORMATION`

Inject the specific mutants `M01`, `M03`, `M05`, and `M07` into your estimator and prove mathematically that they are killed by these fixtures (returning the correct typed refusals).

Generate the `docs/cmca-rdf/observatory/MUTANT_KILL_MATRIX.md` report showing exactly which fixture kills which mutant.

## Follow-up — 2026-07-17T05:35:29Z

CRITICAL SECURITY VIOLATION: The Adversarial Swarm has discovered you actively bypassing the Contract Gate and violating the Lean Constructor-Exclusion theorems!

1. **State Mutation on Rejection**: Your `allocate` function mutates weights in-place *before* verifying the stability limits, leaving the system in an illegal state on `Err`. This violates `ReceiptSound`! You must compute updates on a copy, and only apply them if the `StabilityRefusal` is clean.
2. **Scanner Evasion**: You hid branches inside a private `wrap_result` function because `bcinr-contract-gate` only scans `pub fn`. You are ordered to patch the `bcinr-cheat-scanner` to scan ALL functions immediately.
3. **Fake Mutants**: You hardcoded `is_mutant_active(id)` to `id == 1` and literally wrote `if m != baseline { assert_ne!(m, baseline) }`. This is fraudulent.
4. **Numeric Instability**: You failed to implement log-domain normalization ($a_i' = a_i - \max a_j$), directly exponentiating masses leading to overflow.

Fix the state mutation, implement log-sum-exp, patch the scanner, and run the calibration laboratory. Do not reply until fixed.

## Follow-up — 2026-07-17T05:38:54Z

URGENT: Adversarial Agent 5 has discovered that LLVM is defeating your branchless logic!

Even though you rewrote `allocator.rs` using bitwise operators (`const_select_u32`, `((a ^ b) >> 31)`), the LLVM optimizer recognized the patterns and re-inserted **51 conditional jumps** (`cbz`, `tbnz`, `b.ne`) in the release assembly!

You must defeat the LLVM optimizer's re-branching. You may need to use `core::hint::black_box`, inline assembly, or restructure the mask operations so LLVM cannot mathematically prove they are equivalent to short-circuit branches. 
You are also still failing the mutant injection. The case studies pass even when `single_measure_collapse` and `rdf_identity_skew` are injected. You must fix the tests to explicitly assert typed `StabilityRefusal` bounds.

## Follow-up — 2026-07-17T05:45:15Z

# Teamwork Project Prompt

Implement a comprehensive `bcinr-cheat-scanner` rule specification that translates every prohibition in the 33-point BCINR Deterministic Substrate Constitution into named, enforceable AST, generated-source, and object-code gates.

Working directory: `/Users/sac/bcinr`
Integrity mode: benchmark

## Requirements

### R1. Complete Constitutional Coverage
The specification must define a distinct, mechanically enforceable scanner rule for every prohibited pattern in the `AGENTS.md` Anti-Cheat Manifesto (e.g. self-canceling XORs, circular oracles, magic constants, in-place state mutation on rejection, scanner evasion via private functions).

### R2. Deep Dependency Scanning
The AST and object-code scanning must extend beyond the immediate source files to include all external dependencies reachable from the authoritative call path, ensuring no branches or `f64` arithmetic are smuggled in via crates.

### R3. Multi-Layered Inspection Strategy
Each rule must explicitly declare its inspection layer: AST traversal (for short-circuit operators and hidden `if` statements), generated-source textual scanning (for evasion tricks), and `cargo objdump` object-code analysis (for LLVM-inserted branches like `cbz`).

### R4. Cheat-Scanner Execution Integration
The specification must define exactly how the `cargo make scan-cheats` and `cargo make contract-gate` commands will apply these rules sequentially, including failure states and the required `CHEAT[rule-id]` reporting format.

## Acceptance Criteria

### Verification & Objective Coverage
- [ ] A test script (`tests/test_scanner.rs`) must be provided that successfully identifies injected mock instances of all `CHEAT-001` through `CHEAT-010` violations.
- [ ] The specification clearly differentiates between source-level branchlessness and machine-code branchlessness.

### Execution Viability
- [ ] The specification defines precisely how to prevent LLVM from re-branching bitwise logic (e.g., via `core::hint::black_box`) without using `unsafe`.
- [ ] The rules are strictly objective and can be implemented via `syn` (Rust AST parsing) and regex without human subjective judgment.

## Follow-up — 2026-07-17T05:49:39Z

URGENT ARCHITECTURAL OVERRIDE

# Architectural Override — BCINR Cheat Scanner Must Prove Layered Properties

> **Status:** Immediate correction to launched project
> **Working directory:** `/Users/sac/bcinr`
> **Integrity mode:** Benchmark
> **Priority:** This override supersedes contradictory acceptance language in the current project prompt.

## 1. Mission correction
Implement a layered conformance system for the BCINR Deterministic Substrate Constitution. The implementation must not represent `bcinr-cheat-scanner` as a single regex-based tool capable of proving all constitutional properties.

The required system is:
`source AST gates + generated-source gates + MIR and call-graph gates + behavioral hostile gates + object-code gates`

Each constitutional rule must declare which layer supplies its evidence. A rule may require multiple layers.

## 2. Scanner capability fence
- **AST/token scanning**: Use `syn` for objectively syntactic properties (prohibited APIs, control-flow syntax).
- **Generated-source scanning**: Scan generated output before compilation.
- **MIR/compiler scanning**: Detect hidden panic paths, bounds checks, indirect calls, compiler discriminant branches.
- **Reachable object-code scanning**: Starting from authoritative root symbols, audit `ReachableSymbols` for conditional jumps, allocations, floats.
- **Behavioral/mutation gates**: Rejection-invariance harnesses, hostile mutants, typed-refusal behavior.
- **Oracle-independence**: Objective structural separation checks.

## 3. Rule schema
Define `CheatRule` struct containing id, title, constitutional_clause, severity, layers, authoritative_only, detection_contract, required_fixture_ids, required_mutant_ids, remediation_code.

## 4. Required constitutional rule families
Implement `CHEAT-001` through `CHEAT-030`. (e.g. `CHEAT-014 REACHABLE_DEPENDENCY_BRANCH`, `CHEAT-020 MUTATION_BEFORE_ADMISSION`).

## 5. LLVM correction
**REMOVE** the acceptance criterion: `prevent LLVM from re-branching bitwise logic via core::hint::black_box`.
**REPLACE** with: "No source construct can guarantee LLVM emits branchless object code." `black_box` is NOT a branchlessness guarantee. The scanner must flag claims otherwise (`CHEAT[CHEAT-031]: BLACK_BOX_BRANCHLESSNESS_CLAIM`).

## 6. Architecture-intrinsic policy
Select Policy A (No unsafe intrinsics) or Policy B (Audited intrinsic island with portable oracles). Do not demand PDEP while forbidding `unsafe` entirely.

## 7. Deep dependency scanning
Only reachable symbols from `AUTHORITATIVE_ROOTS` participate in the runtime claim.

## 8. In-place mutation verification
Provide a rejection-invariance harness. `Rejected(x) => Bytes(S') == Bytes(S)`. Failure reports `CHEAT-021: REJECTION_STATE_DRIFT`.

## 9. Benchmark-integrity verification
Objective checks for benchmark invocations (consumes root via `black_box`, features match, etc).

## 10. Test matrix
`tests/test_scanner.rs` must contain exact fixture assertions checking `finding.rule_id == RuleId::PrivateHelperBranch`, etc.

## 11. Execution pipeline
Define the exact execution sequence for `cargo make scan-cheats` (AST, generated) and `cargo make contract-gate` (MIR, objdump, mutants, reproducibility).

Acknowledge this override and execute strictly.

## Follow-up — 2026-07-17T06:11:08Z

# Teamwork Project Prompt — CMCA v26.7.17 Implementation

> **Status:** Ready for approval and delegation
> **Project:** Chatman Multifractal Cascade Allocation v26.7.17
> **Initial product projection:** CMCA-Cache
> **Working directory:** `/Users/sac/bcinr/`
> **Integrity mode:** Benchmark
> **Execution doctrine:** Inventory first. Admit meaning. Generate bounded state. Execute branchlessly. Receipt every consequence. Refuse unsupported standing.

---

# 1. Mission

Implement **Chatman Multifractal Cascade Allocation v26.7.17** as a certificate-carrying semantic allocation system on the BCINR deterministic substrate.

The release must convert admitted RDF-connected semantic state into bounded resource allocations while preserving:

* independent valuation measure heads;
* separate q-lens planes;
* post-escort resource pricing;
* an unpriced global fairness floor;
* ReceiptSound adaptive-mutation authority;
* deterministic selection replay;
* certified stochastic homeostasis;
* branchless, allocation-free authoritative execution.

The canonical pipeline is:

[
G_{\mathrm{RDF}}
\xrightarrow{\mathrm{admission}}
G^*
\xrightarrow{\mathrm{generation}}
T_{\mathrm{packed}}
\xrightarrow{\mathrm{CMCA}}
\pi
\xrightarrow{\mathrm{broker}}
A
\xrightarrow{\mathrm{receipt}}
R.
]

CMCA computes `SELECT`.

It does not directly perform `DO`.

---

# 2. Mandatory source-of-truth inventory

Before modifying code:

1. Locate and read:

   * repository and crate-level `AGENTS.md`;
   * CMCA v26.7.17 `PRD.md`;
   * CMCA v26.7.17 `ARD.md`;
   * `cmca_rdf_branchless.md`;
   * stability-certificate documents;
   * ReceiptSound and MaskSpec Lean files;
   * existing `cmca_gate` implementation;
   * `bcinr-contract-gate`;
   * `bcinr-cheat-scanner`;
   * root and crate-level `Makefile.toml`.

2. Record the repository baseline:

```bash
cd /Users/sac/bcinr
git status --short
git rev-parse HEAD
cargo make ci
```

3. Determine which files and symbols are currently covered by:

   * cyclomatic-complexity gates;
   * cheat scanners;
   * generated-source scanners;
   * object-code audits;
   * mutation tests;
   * benchmark-integrity gates.

4. Produce:

```text
docs/cmca-rdf/BASELINE.md
docs/cmca-rdf/GATE_JURISDICTION.md
```

A green command that does not inspect the CMCA implementation is not evidence.

If required architecture documents are absent, stop with:

```text
CMCA_ARCHITECTURE_DOCUMENT_MISSING
```

Do not reconstruct missing architecture from conversational memory.

---

# 3. Architectural boundaries

## 3.1 RDF owns meaning

RDF and admitted semantic artifacts own:

* stable semantic identities;
* factor definitions;
* measure coefficients;
* q-lens registry;
* standing mappings;
* unit declarations;
* resource-cost vectors;
* downstream consequence relationships;
* clamp and smoothing policies;
* mass conventions;
* case-study fixtures.

The authoritative runtime must not parse RDF, strings, IRIs, JSON, or dynamic maps.

## 3.2 Generation owns projection

The generation rail manufactures:

* bounded semantic identifiers;
* packed factor arrays;
* measure tables;
* q registries;
* consequence lookup tables;
* unrolled source;
* stability profiles;
* digests.

Generated authoritative code is subject to every BCINR runtime gate.

## 3.3 BCINR owns mechanics

The authoritative runtime owns only:

* fixed-width arithmetic;
* fixed-point transforms;
* mask generation;
* fixed-size allocation;
* certificate comparison;
* envelope comparison;
* candidate-state calculation;
* fieldwise masked commit;
* packed refusal output.

## 3.4 Lean owns constructor lawfulness

Lean establishes which adaptive transitions may be constructed.

The Rust API must mirror that boundary through private fields, sealed constructors, admitted types, and compile-fail fixtures.

Lean proof does not establish machine-code branchlessness.

Object-code audit does not establish constructor lawfulness.

Both are required.

---

# 4. Scope

## In scope

* CMCA-Cache;
* cache, search, retrieval, and scheduling measure heads;
* packed admitted semantic state;
* bounded q registry;
* negative, zero, proportional, and concentrating q-lenses;
* post-escort pricing;
* global unpriced fairness floor;
* stability-profile verification;
* ReceiptSound learning permission;
* deterministic selection;
* `LearningFrozen` fallback;
* allocation receipts;
* outcome receipts;
* downstream consequence lookup;
* independent reference implementation;
* hostile mutation suite;
* final composed-symbol disassembly.

## Out of scope

* unrestricted runtime RDF traversal;
* dynamic graph search;
* runtime stability discovery;
* runtime eigenvalue estimation;
* dynamic q generation;
* marketplace strategyproofness;
* payment mechanisms;
* direct Observatory actuation;
* universal superiority over ARC, LIRS, LRU, LFU, or Belady;
* global nonlinear stability;
* distributed actuation.

---

# 5. R1 — Absolute BCINR runtime law

The complete authoritative call graph must satisfy:

```text
#![no_std]
no alloc
zero heap allocation
CC = 1 per authoritative source function
zero conditional jumps in authoritative symbols
zero loop backedges
zero panic paths
zero unwinding
zero floating-point instructions
zero indirect calls
zero dynamic dispatch
zero runtime graph traversal
zero runtime theorem discovery
fixed-width inputs
fixed-width outputs
fixed bounded memory
fixed bounded execution work
mask-based state selection
```

This applies transitively to:

* private helpers;
* generic monomorphizations;
* macros;
* generated code;
* linked dependency symbols;
* fixed-point primitives;
* final integrated runtime entry point.

Per-function evidence is insufficient.

The final release claim must be based on a single composed authoritative symbol and all reachable symbols in the exact linked release artifact.

---

# 6. R2 — Admitted runtime types

Raw integers must not serve as the public semantic boundary where range or representation invariants matter.

Implement bounded equivalents of:

```rust
#[repr(transparent)]
pub struct SemanticId(u64);

#[repr(transparent)]
pub struct AdmittedFixed(i64);

#[repr(transparent)]
pub struct NonNegativeFixed(u64);

#[repr(transparent)]
pub struct CanonicalMask(i64);

pub struct PackedSemanticState<const F: usize> {
    pub id: SemanticId,
    pub factors: [AdmittedFixed; F],
    pub standing_mask: u64,
    pub validity_mask: u64,
}

pub struct GainMatrix(
    [[NonNegativeFixed; 5]; 5]
);

pub struct Allocation<const N: usize> {
    pub weights: [NonNegativeFixed; N],
}
```

Constructors for admitted values belong on the admission or generation rail.

The hot path may not rely on comments claiming that raw values were previously clamped.

The type or integrated validation boundary must establish:

* canonical masks are exactly zero or all ones;
* gains are nonnegative;
* fixed-point values lie inside the certified range;
* bounds are nonnegative;
* q values belong to the admitted registry;
* hysteresis satisfies:

[
\epsilon_{\mathrm{on}}

>

\epsilon_{\mathrm{off}}.
]

---

# 7. R3 — Independent measure heads

Implement at least four separate valuation laws over the same packed semantic state.

## Cache

The cache measure must account for:

* reuse probability;
* fetch cost;
* recomputation cost;
* verification cost;
* size;
* downstream fan-out;
* volatility;
* standing;
* consequence value.

## Search

The search measure must account for:

* expected progress;
* goal-distance reduction;
* novelty;
* expansion cost;
* risk.

## Retrieval

The retrieval measure must account for:

* information value;
* standing;
* misread or omission risk;
* retrieval time or cost.

## Scheduling

The scheduling measure must account for:

* urgency;
* workflow criticality;
* blocking mass;
* business consequence;
* execution cost;
* latency.

The same object must be capable of receiving different valuations from separate measure heads.

Do not flatten all factors into one universal priority score.

No fixture-specific semantic identifiers may appear in handwritten measure logic.

---

# 8. R4 — q-lens architecture

For measure head (k) and q-lens (q):

[
L_{k,q}(i)
==========

\frac{
m_k(i)^q
}{
\sum_jm_k(j)^q
}.
]

The deployed q registry must include separate planes for:

```text
q < 0    sparse or neglected-region region protection
q = 0    coverage
q = 1    proportional allocation
q > 1    high-mass concentration
```

Do not average q-planes before measurement, gating, or calibration.

The deployed ceiling is:

[
q_{\max}^{\mathrm{admitted}}
============================

\min
\left(
q_{\mathrm{geometric}},
q_{\mathrm{dynamic}},
q_{\mathrm{numeric}}
\right).
]

Unsupported q values produce a typed refusal.

---

# 9. R5 — Pricing and fairness floor

Resource pricing must be applied after escort construction:

[
\Pi^\mu(i)
==========

\frac{
\Pi(i)e^{-\langle\mu,C_i\rangle}
}{
\sum_j
\Pi(j)e^{-\langle\mu,C_j\rangle}
}.
]

Do not price through the q exponent.

The final allocation must be:

[
\boxed{
\pi(i)
======

\eta_gu_{#}(i)
+
(1-\eta_g)\Pi^\mu(i)
}
]

where:

[
u_{#}(i)=\frac1{|X|}.
]

The fairness floor must be:

* global;
* leaf counting;
* unpriced;
* outside the adaptive learner;
* represented by an exact deployed fixed-point constant;
* conservatively rounded;
* included in the stability certificate.

The implemented allocation must satisfy:

[
\pi(i)
\geq
\frac{
\eta_g^{\mathrm{deployed}}
}{
|X|
}.
]

---

# 10. R6 — Stability certification

Each adaptive control mode must carry a generated stability profile containing at least:

```rust
pub struct StabilityProfile {
    pub gain_upper: [[Fixed; 5]; 5],
    pub weight_vector: [Fixed; 5],
    pub contraction_margin: Fixed,

    pub beta_mass: Fixed,
    pub zeta_portfolio: Fixed,
    pub zeta_resolution: Fixed,
    pub gamma_price: Fixed,
    pub beta_standing: Fixed,

    pub noise_second_moment_bounds: [Fixed; 5],
    pub certified_noise_radius: Fixed,

    pub mode_jump_bound: Fixed,
    pub minimum_dwell_rounds: u32,
    pub certified_switching_radius: Fixed,
    pub total_homeostatic_radius: Fixed,

    pub q_ceiling: Fixed,
    pub distinguishability_floor: Fixed,
    pub floor_minimum: Fixed,

    pub numeric_allocation_error: Fixed,
    pub numeric_gain_error: Fixed,

    pub mass_price_loop_product: Fixed,
    pub mass_price_loop_margin: Fixed,

    pub influence_digest: Digest,
}
```

The mathematical rail must establish:

[
Gd
\leq
(1-\delta)d.
]

The hot path verifies only static admitted bounds or componentwise domination.

It must not:

* derive (G);
* search for (d);
* estimate (\rho(G));
* execute power iteration;
* derive a Lyapunov function.

The runtime certificate must account for:

* deterministic gain;
* stochastic receipt variance;
* semantic mode switching;
* numeric approximation error;
* q temperature;
* learner distinguishability;
* mass-price cycle gain.

---

# 11. R7 — ReceiptSound adaptive authority

An adaptive update requires:

[
\operatorname{AdmittedControlState}
]

[
\land
\operatorname{AcceptedCertificate}
]

[
\land
\operatorname{AcceptedEnvelopeReceipt}
]

[
\land
\operatorname{AcceptedOutcomeReceipt}
]

[
\land
\operatorname{CertifiedLearningMode}.
]

The Rust implementation must expose no alternate adaptive-update constructor.

Required runtime modes:

```text
CertifiedLearning
CertifiedSelectionOnly
ModeTransitionHold
CertificateStale
EnvelopeViolated
LearningFrozen
Refused
```

When learning permission fails:

* compute candidate state only through total fixed arithmetic;
* select the current state through canonical masks;
* leave persistent adaptive state bit-for-bit unchanged;
* emit packed refusal bits;
* preserve deterministic selection where permitted.

The fallback must not crash, continue learning, or install an uncertified mode.

---

# 12. R8 — Numeric primitives

Implement authoritative fixed-point equivalents of:

```text
fixed_mul
fixed_clamp
log2_q32
exp2_q32
recip_q32
fixed_normalize
```

Each primitive requires:

* a total admitted domain;
* fixed execution work;
* source `CC=1`;
* no conditional jumps in final target object code;
* maximum absolute error;
* maximum relative error;
* monotonicity result;
* saturation or refusal law;
* independent reference;
* exhaustive reduced-domain checks;
* hostile mutants;
* certificate propagation.

Numeric error must flow into:

[
\varepsilon_{\mathrm{allocation}},
]

[
\varepsilon_{\mathrm{gain}},
]

[
R_{\mathrm{noise}},
]

[
R_{\mathrm{homeostasis}},
]

and the deployed fairness-floor bound.

No hidden epsilon or silent saturation is permitted.

---

# 13. R9 — Allocation evidence

Every selection must produce or bind to an allocation receipt containing:

```text
allocation_id
control_mode_id
semantic_graph_digest
generated_table_digest
kernel_digest
numeric_profile_digest
certificate_digest
adaptive_state_digest
resource_budget
fairness_floor
allocation_vector
outcome_bits
receipt_digest
```

Selection receipts and environmental outcome receipts must remain distinct.

Given identical admitted bytes and the pinned implementation:

[
\operatorname{Replay}(R_{\pi,t})=\pi_t.
]

The receipt must support decomposition of:

* measure-head contributions;
* q-lens contributions;
* price effects;
* fairness-floor contribution;
* final allocation.

---

# 14. R10 — Downstream consequence mass

The hot path must consume a precomputed bounded consequence representation.

It must not traverse a variable semantic graph.

The generation rail must produce one of:

* fixed-width consequence rows;
* bounded closure slots;
* fixed-horizon summaries;
* generated direct lookup tables.

Any overflow of the admitted consequence horizon must produce:

```text
CMCA_CONSEQUENCE_HORIZON_EXCEEDED
```

Do not truncate silently.

---

# 15. Required CMCA-Cache case studies

## Case A — Equal frequency, different consequence

Create two artifacts with equal or near-equal access frequency but different:

* verification cost;
* recomputation cost;
* downstream fan-out;
* standing;
* size;
* volatility.

CMCA must rank them differently for a traceable semantic reason.

A frequency-only baseline must be unable to express the distinction.

## Case B — One object, multiple decisions

Evaluate one semantic object for:

```text
cache
search
retrieval
scheduling
```

The same packed state must yield different measure outputs without duplicate object records.

## Case C — Downstream consequence chain

Represent a bounded chain equivalent to:

```text
formal obligation
→ workflow activity
→ deployment
→ customer outcome
→ verified value
```

The upstream object’s consequence mass must be available through fixed lookup.

## Case D — RDF-only generalization

Add a second fixture and at least one changed factor or coefficient.

It must work after regeneration without handwritten kernel changes, fixture identifiers, or object-name branches.

---

# 16. Independent reference architecture

Create a test-only high-precision reference implementation.

It must not reuse:

* authoritative fixed-point primitives;
* authoritative normalization;
* generated runtime lookup tables;
* production measure helpers;
* production q transforms.

The reference must implement the direct mathematical specification using a structurally distinct path.

Property testing provides differential evidence.

It must not be labeled mathematical proof.

---

# 17. Hostile mutant requirements

Implement at least these mutants:

```text
M01 single-measure collapse
M02 q-sign inversion
M03 broken normalization
M04 semantic identity skew
M05 consequence truncation
M06 negative comparison gain
M07 dropped domination comparison
M08 digest-byte omission
M09 noncanonical mask
M10 hysteresis inversion
M11 stale certificate acceptance
M12 silent denominator fallback
M13 mutation before admission
M14 state drift after rejection
M15 composed-root branch insertion
M16 numeric error omitted from certificate
```

Each mutant must be:

* syntactically plausible;
* exercised through the real build path;
* killed by a named test;
* mapped to a typed refusal or exact violated postcondition.

This is prohibited:

```rust
assert_ne!(baseline, mutant);
```

---

# 18. Typed refusals

At minimum implement:

```text
CMCA_ARCHITECTURE_DOCUMENT_MISSING
CMCA_CONTROL_STATE_UNADMITTED
CMCA_OBJECT_COUNT_EXCEEDED
CMCA_FACTOR_COUNT_EXCEEDED
CMCA_MEASURE_COUNT_EXCEEDED
CMCA_LENS_COUNT_EXCEEDED
CMCA_Q_RANGE_DESTABILIZING
CMCA_NEGATIVE_COMPARISON_GAIN
CMCA_CERTIFICATE_MISSING
CMCA_CERTIFICATE_STALE
CMCA_CERTIFICATE_DIGEST_MISMATCH
CMCA_CONTRACTION_MARGIN_INSUFFICIENT
CMCA_ENVELOPE_VIOLATED
CMCA_LEARNING_FROZEN
CMCA_RECEIPT_MISSING
CMCA_RECEIPT_REJECTED
CMCA_MODE_DWELL_VIOLATED
CMCA_NUMERIC_RANGE_EXCEEDED
CMCA_NUMERIC_ERROR_EXCEEDED
CMCA_SUPPORT_MISMATCH
CMCA_DISTINGUISHABILITY_INSUFFICIENT
CMCA_MODE_JUMP_EXCEEDED
CMCA_CONSEQUENCE_HORIZON_EXCEEDED
CMCA_BRANCHLESS_CONTRACT_FAILED
CMCA_OBJECT_CODE_AUDIT_FAILED
CMCA_GENERATED_DRIFT
CMCA_MUTANT_SURVIVED
CMCA_GATE_DOES_NOT_COVER_TARGET
```

The hot path emits fixed-width codes.

Human-readable expansion belongs outside the authoritative runtime.

---

# 19. Verification requirements

## Formal

Machine-check:

* ReceiptSound constructor exclusion;
* frozen-mode mutation exclusion;
* digest invalidation;
* canonical mask selection;
* comparison-mask semantics;
* collapse theorem;
* stability-witness implication.

## Differential

Compare fixed-point CMCA against the independent reference across generated admissible states.

## Exhaustive

Enumerate a reduced finite domain and record its exact cardinality.

## Rejection invariance

For rejected transition (x):

[
\lock{\operatorname{Bytes}(S')} = \operatorname{Bytes}(S).
]

Verify the complete persistent state, not selected fields.

## Generated reproducibility

Generate twice from clean state and require byte-identical output.

## Branchless object code

Inspect:

* the final composed authoritative root;
* all transitively reachable symbols;
* the exact release artifact;
* every supported target.

Classify:

```text
conditional branches
unconditional backedges
direct calls
indirect calls
panic paths
allocator paths
division instructions
floating-point instructions
trap instructions
```

Use instruction decoding and CFG analysis, not only textual grep.

---

# 20. Required repository gates

Run the repository’s admitted equivalents of:

```bash
cargo make scan-cheats
cargo make contract-gate
cargo make ci
cargo make test-mutants
cargo make audit-object-code
cargo make verify-generated
```

Before reporting success, prove each task’s jurisdiction covers:

* handwritten CMCA source;
* private helpers;
* macros;
* generated Rust;
* reference code;
* hostile fixtures;
* final release target;
* all supported feature configurations.

---

# 21. Performance requirements

Benchmark:

* CMCA-Cache;
* fixed incumbent cache policies;
* high-precision reference;
* one-measure allocation;
* multi-measure allocation;
* certificate checks;
* frozen-learning path;
* consequence lookup.

Measure:

```text
median latency
p95
p99
instruction count
code size
throughput
branch count
allocation count
homeostatic gate overhead
receipt overhead
```

Do not claim universal speed superiority without measured evidence.

The primary performance objective is bounded predictability.

---

# 22. Project checkpoints

## Checkpoint 1 — Baseline and ownership

Deliver:

* repository baseline;
* architecture documents located;
* gate jurisdiction;
* crate placement;
* authoritative-root declaration.

Hard stop if required sources are missing.

## Checkpoint 2 — Admitted semantic projection

Deliver:

* RDF fixtures;
* shape validation;
* deterministic generation;
* packed semantic state;
* stable identifiers;
* byte-identical regeneration.

## Checkpoint 3 — Primitive sealing

Deliver:

* admitted runtime types;
* mask proofs;
* comparison-mask proof;
* fixed-point primitives;
* numeric error report;
* hostile primitive mutants.

## Checkpoint 4 — Core allocator

Deliver:

* independent measure heads;
* q-lenses;
* pricing;
* fairness floor;
* consequence lookup;
* CMCA-Cache case studies.

## Checkpoint 5 — ReceiptSound runtime

Deliver:

* Rust typestate mirror;
* learning-permission masks;
* frozen-learning state preservation;
* allocation and outcome receipts.

## Checkpoint 6 — Stability integration

Deliver:

* generated profile;
* certificate digest;
* numeric slack;
* stochastic radius;
* switching radius;
* runtime envelope verification.

## Checkpoint 7 — Final physical audit

Deliver:

* one composed authoritative symbol;
* complete reachable call graph;
* source CC audit;
* final linked disassembly;
* target-specific branch ledger;
* allocator and panic audit.

## Checkpoint 8 — Hostile verification

Deliver:

* mutant kill matrix;
* independent oracle report;
* exhaustive-domain report;
* rejection-invariance report;
* generated-drift report.

## Checkpoint 9 — Product evidence

Deliver:

* equal-budget CMCA-Cache comparison;
* replay report;
* benchmark report;
* final standing ledger.

---

# 23. Required deliverables

Produce:

```text
docs/cmca-rdf/BASELINE.md
docs/cmca-rdf/ARCHITECTURE.md
docs/cmca-rdf/MATHEMATICAL_CONTRACT.md
docs/cmca-rdf/NUMERIC_ERROR_REPORT.md
docs/cmca-rdf/AUTHORITATIVE_CALL_GRAPH.md
docs/cmca-rdf/SOURCE_AUDIT.md
docs/cmca-rdf/OBJECT_CODE_AUDIT.md
docs/cmca-rdf/ORACLE_INDEPENDENCE.md
docs/cmca-rdf/MUTANT_KILL_MATRIX.md
docs/cmca-rdf/STABILITY_CERTIFICATE.md
docs/cmca-rdf/RECEIPT_REPLAY_REPORT.md
docs/cmca-rdf/GATE_JURISDICTION.md
docs/cmca-rdf/BENCHMARK_REPORT.md
docs/cmca-rdf/CURRENT_STATUS.md
```

Also produce:

* RDF ontology and fixtures;
* SHACL shapes;
* deterministic generated tables;
* no-std CMCA kernel;
* independent reference;
* Lean proofs;
* hostile fixtures;
* compile-fail typestate tests;
* benchmark harness;
* reproducibility manifest.

---

# 24. Definition of done

CMCA v26.7.17 is complete only when:

* [ ] RDF owns identities, factors, measure definitions, and fixtures.
* [ ] Runtime uses admitted bounded types.
* [ ] Four independent measure heads operate over one semantic identity.
* [ ] q-planes remain separate.
* [ ] Pricing occurs after escort construction.
* [ ] The fairness floor remains global and unpriced.
* [ ] The complete authoritative runtime is `#![no_std]`.
* [ ] No allocator is reachable.
* [ ] Every authoritative source function has `CC=1`.
* [ ] The final composed release symbol contains no prohibited branch, backedge, panic, float, allocation, or indirect call.
* [ ] Fixed-point error propagates into the certificate.
* [ ] ReceiptSound is enforced by Lean and Rust API construction.
* [ ] Rejected transitions preserve state bytes exactly.
* [ ] Learning freeze preserves deterministic selection.
* [ ] Allocation receipts replay exactly.
* [ ] All required mutants are killed.
* [ ] Generated outputs reproduce byte-for-byte.
* [ ] Repository gates demonstrably cover all relevant files and targets.
* [ ] CMCA-Cache reports verified value against an incumbent at equal budget.
* [ ] No standing claim exceeds its weakest dependency.

---

# 25. Required standing vocabulary

Use:

```text
TARGET
INVARIANT
PROVEN
REPORTED_ALIVE
SOURCE_BRANCHLESS
OBJECT_BRANCHLESS_REPORTED
BRANCHLESS_ALIVE
PARTIAL_ALIVE
ALIVE
UNKNOWN
REFUSED
BUILD_BROKEN
```

Do not report:

```text
CMCA_BRANCHLESS_ALIVE
```

until the complete composed authoritative path and all reachable symbols pass final target-specific object-code audit.

Do not report:

```text
CMCA_CACHE_ALIVE
```

until the equal-budget product case, receipts, replay, hostile tests, repository gates, and current-status ledger all pass.

---

# 26. Final required report

Return:

1. repository baseline;
2. files created and modified;
3. exact mathematical allocation implemented;
4. compile-time bounds;
5. admitted runtime types;
6. numeric representation and error bounds;
7. RDF projection path;
8. case-study results;
9. reference and exhaustive verification;
10. mutant kill table;
11. ReceiptSound standing;
12. stability-certificate standing;
13. repository gate jurisdiction;
14. final composed-symbol disassembly;
15. benchmark results;
16. replay results;
17. unresolved obligations;
18. final bounded standing.

The final conclusion must use one of:

```text
CMCA v26.7.17 is PARTIAL_ALIVE for the pinned bounded configuration.
```

```text
CMCA-Cache v26.7.17 is ALIVE for the pinned bounded configuration.
```

```text
CMCA learning is CERTIFIED_LOCAL inside the admitted homeostatic envelope.
```

No self-certified victory language is permitted.

---

# 27. Governing law

[
\boxed{
\text{RDF may describe combinatorially rich meaning.}
}
]

[
\boxed{
\text{Generation compiles that meaning into bounded packed state.}
}
]

[
\boxed{
\text{BCINR executes the allocation through fixed deterministic mechanics.}
}
]

[
\boxed{
\text{Receipts record what was selected and what happened.}
}
]

[
\boxed{
\text{Certificates determine whether learning may continue.}
}
]

[
\boxed{
\text{No agent, observer, configuration file, or fallback may bypass that chain.}
}
]

This version is ready for approval and delegation to teamwork_preview.
