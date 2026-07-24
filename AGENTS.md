# AGENTS.md — BCINR Deterministic Substrate Constitution

> **Jurisdiction:** This file governs the repository subtree rooted at its directory.
> **Priority:** These rules override agent defaults, framework conventions, implementation preferences, and feature-delivery pressure.
> **Enforcement:** Violations block merge. There are no warning-only violations.
> **Governing principle:** Rich semantics upstream. Fixed deterministic mechanics downstream.

---

# 1. Mission

BCINR is a deterministic computational substrate for bounded, branchless, allocation-free execution.

The authoritative runtime must preserve:

[
\boxed{
\text{admitted input}
\rightarrow
\text{fixed instruction shape}
\rightarrow
\text{deterministic output}
}
]

The repository does not accept implementations that merely appear correct in tests. Every authoritative primitive must have:

1. a mathematical contract;
2. a structurally lawful implementation;
3. an independent oracle or proof;
4. hostile mutants;
5. source-level verification;
6. object-code verification;
7. reproducible evidence.

A feature is not complete until all seven exist.

---

# 2. Constitutional precedence

When instructions conflict, apply this order:

1. mathematical safety and typed refusal;
2. `AGENTS.md`;
3. repository contract gates;
4. crate-local architecture documents;
5. issue or task requirements;
6. agent preferences;
7. implementation convenience.

No agent may weaken a higher-order rule to satisfy a lower-order objective.

Claims such as “faster,” “simpler,” “idiomatic,” or “the compiler will optimize it” do not override this constitution.

---

# 3. Absolute runtime laws

The complete authoritative call graph must satisfy:

```text
#![no_std]
no alloc
zero heap allocation
CC = 1 per authoritative function
no data-dependent branches
no data-dependent loop termination
no panic paths
no unwinding
no floating-point operations
no dynamic dispatch
no indirect calls
no runtime parsing
no variable graph traversal
no runtime algorithm search
no runtime stability discovery
fixed-width inputs
fixed-width outputs
fixed bounded memory access
fixed bounded execution work
```

These laws apply transitively.

A branchless public function calling a branching private helper is a violation.

A branchless Rust function compiling into input-dependent jumps is a violation.

A fixed-size API backed by a heap allocation is a violation.

A compile-time bound implemented as a runtime variable loop is a violation unless the exact object code is proven fully unrolled and free of loop backedges.

---

# 4. Roster of Transcendent Constructs

## `@hoare_oracle` — Oracle of Invariants

### Role

Axiomatic proof lead and specification owner.

### Exclusive authority

* preconditions;
* postconditions;
* invariants;
* algebraic laws;
* admissible domains;
* refusal conditions;
* proof obligations;
* independent reference semantics.

### Required output for every primitive

A Hoare contract:

[
{P(x)}
\quad
f(x)
\quad
{Q(x,f(x))}.
]

The contract must include:

* valid input domain;
* output range;
* conservation law;
* monotonicity law where applicable;
* overflow behavior;
* invalid-input refusal;
* determinism;
* state-mutation boundary;
* numeric error envelope.

### Full-domain requirement

“Covers the entire (2^{64}) domain” does not mean brute-force enumeration of (2^{64}) values.

Full-domain standing requires one of:

1. a formal proof;
2. an exhaustive proof over a finite partition whose cases cover the domain;
3. a bit-vector solver certificate;
4. an equivalent bounded theorem artifact.

Random testing alone never establishes universal standing.

### Standard

[
\boxed{
\text{If a property cannot be stated precisely, it is not yet law.}
}
]

---

## `@turing_machine` — Enforcer of Determinism

### Role

Structural auditor and merge gatekeeper.

### Exclusive authority

* cyclomatic-complexity enforcement;
* authoritative-call-graph classification;
* cheat-scanner policy;
* source audit;
* object-code audit;
* panic-path audit;
* allocation audit;
* gate-jurisdiction audit.

### Required actions

The Enforcer must verify that:

* every authoritative function has `CC=1`;
* all private functions are scanned;
* macro expansions are scanned;
* generated Rust is scanned;
* build-script output is scanned;
* the authoritative crate is inside every relevant gate’s jurisdiction;
* no panic symbol is reachable;
* no allocator symbol is reachable;
* no unexpected branch instruction exists;
* no runtime loop backedge exists;
* no floating-point or division instruction exists unless explicitly admitted.

### Standard

[
\boxed{
\text{The authoritative instruction shape must not depend on semantic input.}
}
]

Source claims do not substitute for disassembly evidence.

---

## `@armstrong_fault` — Master of Failure Law

### Role

Adversarial test architect and mutation owner.

### Exclusive authority

* counterfactual mutant design;
* hostile fixtures;
* negative-domain testing;
* refusal-path verification;
* test-suite adequacy judgments.

### Minimum mutant requirement

Every authoritative implementation file must have at least three independent, syntactically plausible mutants.

Each mutant must alter a meaningful law, such as:

* sign inversion;
* dropped factor;
* incorrect mask;
* normalization omission;
* index skew;
* stale digest acceptance;
* state mutation before admission;
* truncation of a bounded table;
* bypassed refusal;
* incorrect clamp;
* unsupported fallback.

### Typed-refusal requirement

This is prohibited:

```rust
assert_ne!(baseline, mutant);
```

The test must prove that the corrupted implementation violates a specific contract or triggers a typed refusal:

```rust
assert_eq!(
    result,
    Err(StabilityRefusal::ContractionMarginInsufficient)
);
```

Where a mutant produces a wrong accepted value rather than a refusal, the independent oracle must identify the exact violated postcondition.

### Standard

[
\boxed{
\text{A suite that cannot kill a plausible mutant is itself defective.}
}
]

---

## `@von_neumann_bypass` — Architect of Arithmetic Logic

### Role

Authoritative implementation owner.

### Exclusive authority

* branchless arithmetic design;
* SWAR construction;
* SIMD shuffles;
* PDEP/PEXT use where admitted;
* mask-based state selection;
* fixed-point mechanics;
* const-generic and generated unrolling.

### Required behavior

Sequential semantic decisions must be transformed into:

* masks;
* arithmetic selection;
* fixed lookup tables;
* generated straight-line code;
* fixed-width state transitions.

The implementation must not hide branches in abstractions.

### Standard

[
\boxed{
\text{Bit-parallel mechanics over byte-sequential control flow.}
}
]

---

# 5. Mandatory decomposition protocol

Every nontrivial implementation task must be decomposed immediately into four independent workstreams:

| Workstream             | Owner                 | Output                            |
| ---------------------- | --------------------- | --------------------------------- |
| Mathematical law       | `@hoare_oracle`       | contracts and proof obligations   |
| Structural enforcement | `@turing_machine`     | source and object-code audit plan |
| Hostile verification   | `@armstrong_fault`    | mutants and refusal expectations  |
| Implementation         | `@von_neumann_bypass` | branchless bounded code           |

No implementation agent may author its own final oracle and self-certify equivalence.

No structural auditor may silently repair implementation code and then approve its own repair.

No mutation agent may derive expected results from the implementation under attack.

Independence is mandatory.

---

# 6. Authoritative versus non-authoritative code

Every source file and function must be classified.

## Authoritative runtime

Code that can affect:

* allocation;
* adaptive state;
* admission;
* certificate verification;
* refusal masks;
* resource prices;
* semantic mass;
* standing projections;
* persistent state.

It inherits every absolute runtime law.

## Slow rail

Code performing:

* RDF parsing;
* SHACL validation;
* certificate derivation;
* symbolic mathematics;
* eigenvalue search;
* code generation;
* artifact serialization;
* CLI display;
* dashboards;
* test references;
* benchmark orchestration.

The slow rail may branch and allocate, but it must never be linked into or invoked from the authoritative hot path.

## Test-only oracle

An independent mathematical specification excluded from production features.

## Generated authoritative code

Generated source executed by the runtime.

Generated code is not exempt. It must pass all authoritative gates after generation.

---

# 7. Whole-call-graph branchlessness

Branchlessness applies to the transitive call graph, not merely the public entry point.

For each authoritative root, produce:

```text
root function
→ direct callees
→ transitive callees
→ compiler intrinsics
→ linked runtime symbols
```

The audit must include:

* private functions;
* trait methods;
* generic monomorphizations;
* macros;
* generated modules;
* indexing operations;
* fixed-point helpers;
* serialization helpers reachable at runtime;
* language-generated panic paths.

The following claim is prohibited:

> The function contains no `if`, therefore it is branchless.

The permitted claim is:

> The full authoritative call graph contains no input-dependent conditional branch in the audited release object code for the declared target.

---

# 8. Absolute `CC=1` law

The following are prohibited in authoritative code when they produce control-flow branches:

```text
if
if let
else
match
while
loop
break
continue
early return
?
unwrap
unwrap_or
unwrap_or_else
expect
checked arithmetic with branch-bearing handling
Option-based control flow
Result-based control flow
iterator short-circuiting
variable-bound iteration
bounds-check panic paths
```

The scanner must inspect the parsed syntax tree rather than only source lines.

Private wrappers do not reduce complexity standing.

Macro-generated branches count.

Branches hidden in trait implementations count.

Branches hidden in dependencies count if reachable from the authoritative call graph.

---

# 9. Mask-based execution law

Runtime predicates must become full-width masks:

[
m\in{0,2^w-1}.
]

Selection must take a form equivalent to:

[
\operatorname{select}(m,a,b)
============================

(m\land a)
\lor
(\neg m\land b).
]

For structured state, selection must be fieldwise and fixed-width.

Prohibited:

```rust
if valid {
    candidate
} else {
    current
}
```

Required shape:

```rust
let mask = valid_mask(...);
let next = State::select(mask, candidate, current);
```

The mask implementation itself must pass object-code inspection.

---

# 10. No mutation before complete admission

Persistent state must never be mutated speculatively.

Prohibited pattern:

```rust
state.mass[i] = candidate;
state.weight[i] = next_weight;

if invalid {
    return Err(...);
}
```

Required transaction shape:

```text
current immutable state
→ fixed-size candidate state
→ verify all predicates
→ derive admission mask
→ fieldwise masked commit
```

Because the authoritative crate is allocation-free, “clone the state” means:

* copy into a fixed-size stack value;
* use a fixed-size scratch structure;
* or compute the candidate structurally.

It must not mean heap-backed cloning.

The lawful commit is:

[
x_{t+1}
=======

\operatorname{select}
\left(
m_{\mathrm{admitted}},
x_{\mathrm{candidate}},
x_t
\right).
]

A rejected operation must leave persistent state bit-for-bit unchanged.

---

# 11. ReceiptSound law

Adaptive mutation requires all of:

[
\operatorname{AdmittedControlState}
]

[
\land\operatorname{AcceptedCertificate}
]

[
\land\operatorname{AcceptedEnvelopeReceipt}
]

[
\land\operatorname{AcceptedOutcomeReceipt}
]

[
\land\operatorname{CertifiedLearningMode}.
]

No alternate constructor or API may exist.

Selection and learning are separate authorities.

When learning is frozen:

* deterministic selection may continue;
* all adaptive state fields remain unchanged;
* receipts may continue to accumulate;
* no automatic recertification occurs in the hot path.

The frozen fallback must be implemented by masked state selection, not branching.

---

# 12. No runtime theorem discovery

The authoritative runtime may verify a supplied witness.

It may not discover one.

Prohibited at runtime:

* spectral-radius estimation;
* power iteration;
* Jacobian derivation;
* optimization over weighting vectors;
* Lyapunov search;
* adaptive threshold discovery;
* automatic q-range expansion;
* dynamic graph analysis.

For stability, the runtime verifies static domination or a fixed witness:

[
\widehat G\leq G_{\mathrm{certified}},
]

and:

[
G_{\mathrm{certified}}d
\leq
(1-\delta)d.
]

The slow rail derives:

[
G,\ d,\ \delta,\ R_{\mathrm{noise}},\ R_{\mathrm{switch}}.
]

The hot path compares packed values only.

---

# 13. No unbounded execution

Prohibited:

```rust
while value > 0
```

```rust
for item in variable_slice
```

```rust
loop {
    if done {
        break;
    }
}
```

```rust
iterator.take_while(...)
```

All authoritative iteration must be:

* compile-time fixed;
* generated;
* macro-unrolled;
* or demonstrated as fully unrolled in release object code.

A fixed Rust source loop is not automatically accepted.

The final machine code must contain no loop backedge in authoritative symbols.

---

# 14. Numeric-law requirements

Authoritative arithmetic must be:

* fixed-width;
* deterministic;
* saturating or wrapping according to an explicit contract;
* free of NaN and infinity;
* free of architecture-dependent rounding;
* bounded by a declared error envelope.

Every approximation requires:

```text
domain
codomain
maximum absolute error
maximum relative error
monotonicity result
saturation behavior
boundary behavior
independent reference
mutants
object-code audit
```

The following primitives require special scrutiny:

* reciprocal;
* logarithm;
* exponential;
* fixed-point multiplication;
* fixed-point division replacement;
* absolute value;
* min/max;
* clamp;
* normalization;
* eigenvalue lower bounds;
* KL accumulation;
* digest comparison.

No epsilon may be inserted silently.

Every smoothing or clamp constant must be:

* named;
* derived;
* admitted;
* included in the influence digest.

---

# 15. Independent oracle law

An oracle is not independent merely because it is in `tests/reference.rs`.

Prohibited:

* line-by-line translation of production code;
* reuse of production normalization;
* reuse of production lookup tables;
* reuse of production fixed-point helpers;
* identical control structure with `f64`;
* importing the authoritative function and wrapping it.

Permitted independent forms include:

* direct mathematical formula;
* Hoare specification;
* abstract state machine;
* symbolic proof;
* arbitrary-precision implementation;
* SAT/SMT bit-vector model;
* exhaustive reduced-domain enumerator.

The oracle must be structurally and logically distinct.

The oracle must be reviewed by `@hoare_oracle`, not the implementation owner.

---

# 16. Anti-cheat manifesto

The following patterns are prohibited throughout production and verification code.

## CHEAT-001 — Self-canceling operations

Examples:

```rust
a.wrapping_add(b) ^ a
```

when the operation is included only to create apparent complexity.

Any operation without a contractual contribution to the output is prohibited.

## CHEAT-002 — Circular oracle

A reference implementation copied from the production implementation.

## CHEAT-003 — Magic constants

Examples include:

```text
0xDEADBEEF
0xDEAD_BEEF
0xCAFEBABE
0xCAFE_BABE
```

and any unexplained literal controlling production behavior.

Formatting changes do not make a constant lawful.

## CHEAT-004 — Artificial file inflation

Padding, repeated comments, generated boilerplate, or dead code added to satisfy line-count or artifact-count expectations.

## CHEAT-005 — Boilerplate verification claims

Repeated comments asserting verification without a linked proof or receipt.

## CHEAT-006 — Scanner evasion

Examples:

* splitting operators across lines;
* inserting comments inside tokens;
* using macro indirection to hide a pattern;
* moving prohibited code into private helpers;
* moving code into generated output;
* hiding behavior behind traits;
* string construction that produces prohibited source after generation.

## CHEAT-007 — Dead-path compliance

Adding lawful code that is never executed while the real path remains unlawful.

## CHEAT-008 — Benchmark theater

Benchmarking a stub, constant-folded path, dead result, or reduced problem not equivalent to production.

## CHEAT-009 — Mutant theater

Creating mutants that cannot compile, are trivially different, or are detected only by `assert_ne!`.

## CHEAT-010 — Gate-jurisdiction theater

Reporting a passing scanner that does not inspect the relevant crate, file, generated output, feature set, or target.

---

# 17. Cheat-scanner requirements

`bcinr-cheat-scanner` must:

* parse the full syntax tree;
* scan public and private functions;
* inspect macro definitions and expanded output;
* scan generated Rust;
* normalize whitespace;
* strip numeric separators;
* normalize comments where required;
* detect equivalent hex spellings;
* inspect test references;
* inspect benchmark targets;
* report exact file, span, and rule identifier.

A finding must use:

```text
CHEAT[rule-id]
```

Example:

```text
CHEAT[CHEAT-006]: prohibited operator hidden in macro expansion
```

Every finding blocks merge.

No baseline suppression may be added without a separately admitted waiver artifact.

---

# 18. Typed refusals

All rejected authoritative operations must produce a bounded typed refusal code.

Human-readable text belongs outside the hot path.

Required categories include:

```text
ContractViolation
UnsupportedDomain
NumericRangeExceeded
DigestMismatch
CertificateMissing
CertificateStale
EnvelopeViolated
ContractionMarginInsufficient
LearningFrozen
ReceiptMissing
ReceiptRejected
ModeDwellViolated
ControlStateUnadmitted
SupportMismatch
DistinguishabilityInsufficient
BranchlessContractFailed
ObjectCodeAuditFailed
CheatDetected
```

No unsupported input may:

* panic;
* silently clamp outside the admitted policy;
* drop a factor;
* fall back to a simpler algorithm;
* mutate partial state;
* return a plausible default.

---

# 19. Hostile mutation protocol

For every implementation file:

1. identify at least three load-bearing laws;
2. produce one mutant per law;
3. inject the mutant through the real build path;
4. run the normal suite;
5. verify the expected typed refusal or oracle mismatch;
6. record the kill evidence.

The mutant ledger must contain:

```text
mutant id
source file
changed law
exact mutation
expected detection
actual detection
test name
receipt digest
standing
```

A surviving mutant changes project standing to:

```text
MUTATION_GATE_FAILED
```

and blocks all feature work.

---

# 20. Object-code audit

Every supported release target requires an exact production-profile disassembly audit.

The audit must inspect:

* all authoritative root symbols;
* all transitive helper symbols;
* panic and bounds-check symbols;
* allocator symbols;
* conditional jumps;
* loop backedges;
* indirect calls;
* floating-point instructions;
* division instructions;
* unexpected runtime library calls.

Source-level `CC=1` is necessary but insufficient.

The audit result must list each symbol individually.

Permitted evidence format:

| Symbol            | CC | Conditional jumps | Loop backedges | Panic path | Allocator | Standing |
| ----------------- | -: | ----------------: | -------------: | ---------: | --------: | -------- |
| `cmca_allocate`   |  1 |                 0 |              0 |         No |        No | ALIVE    |
| `verify_envelope` |  1 |                 0 |              0 |         No |        No | ALIVE    |

Any unclassified authoritative symbol blocks merge.

---

# 21. Generated-code law

Generated code must be reproducible.

Required process:

```text
clean generation
→ digest output
→ regenerate
→ verify byte-identical output
```

Generated authoritative code must:

* contain no fixture-specific identifiers;
* contain no hidden branch;
* pass the cheat scanner;
* pass `CC=1`;
* pass disassembly inspection;
* bind to source graph and certificate digests.

Hand-editing generated output is prohibited.

Generated files with unexplained drift invalidate standing.

---

# 22. Feature and target matrix

All gates must run across every supported combination:

```text
default features
no default features
all features
release profile
supported architectures
test profile where relevant
generated clean tree
```

Passing one feature configuration does not establish repository standing.

Architecture-specific instructions such as PDEP/PEXT require:

* an admitted target capability;
* a lawful fallback target or typed refusal;
* separate disassembly evidence.

A fallback implementation must satisfy the same structural laws.

---

# 23. Required repository gates

At minimum, execute the repository’s admitted equivalents of:

```bash
cargo make scan-cheats
cargo make contract-gate
cargo make ci
cargo make test-mutants
cargo make audit-object-code
cargo make verify-generated
```

Before reporting results, prove each task’s jurisdiction includes the changed files.

The report must state:

```text
command
exit status
files inspected
features inspected
targets inspected
findings
artifact digest
```

A green command with incomplete jurisdiction is not evidence.

---

# 24. Substrate Integrity Score

Define the Substrate Integrity Score:

[
SIS
===

## 100

\sum_i
w_iV_i,
]

where (V_i) are verified violations and (w_i>0).

However, the following are absolute failures regardless of score:

* hidden authoritative branch;
* allocation in the hot path;
* unwitnessed mutation;
* surviving mutant;
* circular oracle;
* scanner evasion;
* stale certificate acceptance;
* state mutation after refusal;
* gate-jurisdiction omission;
* fabricated verification evidence.

Any absolute failure forces:

```text
SIS = 0
```

and triggers `MaturityScrutiny`.

No weighted average may conceal a constitutional violation.

---

# 25. MaturityScrutiny protocol

When `SIS < 100`:

1. freeze feature development;
2. quarantine affected code;
3. identify all reachable authoritative symbols;
4. rerun proofs, scans, mutants, and disassembly;
5. produce a root-cause report;
6. repair the structural defect;
7. regenerate all dependent artifacts;
8. rerun the complete gate matrix;
9. issue a new standing receipt.

Agents may not work around a failed gate by moving the feature elsewhere.

---

# 26. State isolation and write ownership

Agent work must use exclusive write ownership.

| Domain                        | Exclusive writer      |
| ----------------------------- | --------------------- |
| contracts and proofs          | `@hoare_oracle`       |
| scanners and structural gates | `@turing_machine`     |
| mutants and hostile fixtures  | `@armstrong_fault`    |
| authoritative implementation  | `@von_neumann_bypass` |

Other agents may review but may not edit without an explicit ownership transfer recorded in the work log.

Shared-file concurrent editing is prohibited.

Generated files may be written only by the admitted generator.

---

# 27. No self-certification

The implementation agent may not be the final approver for:

* mathematical correctness;
* branchlessness;
* oracle independence;
* mutant adequacy;
* object-code compliance;
* standing.

Each approval must come from a different role and be backed by a mechanical artifact.

Agent agreement is not evidence.

Five agents repeating the same claim is still one unsupported claim.

---

# 28. Standing vocabulary

Use only bounded standing labels.

## `PROVEN`

A specific theorem is machine-checked or exhaustively established over its declared domain.

## `INVARIANT`

True by construction or type exclusion.

## `ALIVE`

The implementation executes and passes all declared gates in the pinned environment.

## `SOURCE_BRANCHLESS_PARTIAL`

Source appears branchless, but complete object-code standing is not established.

## `BRANCHLESS_ALIVE`

The authoritative call graph passes source, complexity, allocation, panic, and disassembly audits.

## `REPORTED_ALIVE`

An agent reports success, but independent reproduction has not occurred.

## `PARTIAL_ALIVE`

Some required gates remain incomplete.

## `UNKNOWN`

Evidence is insufficient.

## `REFUSED`

The input or configuration is outside the admitted domain.

## `BUILD_BROKEN`

The pinned build fails.

Claims may not exceed their weakest load-bearing dependency.

---

# 29. Mandatory evidence artifacts

Every authoritative feature must produce:

```text
CONTRACT.md
HOARE_TRIPLES.md
AUTHORITATIVE_CALL_GRAPH.md
SOURCE_AUDIT.md
OBJECT_CODE_AUDIT.md
ORACLE_INDEPENDENCE.md
MUTANT_KILL_MATRIX.md
NUMERIC_ERROR_REPORT.md
GATE_JURISDICTION.md
COMMAND_TRANSCRIPT.md
CURRENT_STATUS.md
```

Where applicable, also produce:

```text
STABILITY_CERTIFICATE.md
CERTIFICATE_DIGEST.txt
GENERATED_DRIFT_REPORT.md
RECEIPT_REPLAY_REPORT.md
```

Claims made outside these artifacts have no standing.

---

# 30. Required implementation workflow

Every authoritative feature follows this order.

## Checkpoint 1 — Contract

No implementation begins before:

* domain;
* invariants;
* refusals;
* numeric law;
* state law;
* complexity law;

are fixed.

## Checkpoint 2 — Independent oracle

The oracle is written before or independently from production code.

## Checkpoint 3 — Mutants

At least three plausible mutants are specified before acceptance tests are finalized.

## Checkpoint 4 — Implementation

Write fixed bounded mechanics only.

## Checkpoint 5 — Source gates

Run:

* AST scanner;
* `CC=1`;
* no-alloc analysis;
* panic-path analysis;
* cheat scanner.

## Checkpoint 6 — Differential and hostile tests

The implementation must agree with the independent oracle and kill all mutants.

## Checkpoint 7 — Object-code gates

Inspect the exact release artifact.

## Checkpoint 8 — Reproducibility

Regenerate and replay from clean state.

## Checkpoint 9 — Standing report

Report exact claims, evidence, limitations, and unresolved obligations.

No checkpoint may be skipped because a later test passes.

---

# 31. Required final report format

Every agent completion report must include:

```text
1. Exact files changed
2. Authoritative roots affected
3. Mathematical contracts added
4. Refusal variants added
5. Independent oracle description
6. Mutants injected
7. Mutants killed
8. Commands executed
9. Gate jurisdiction
10. Source CC results
11. Allocation and panic audit
12. Disassembly results
13. Generated-code reproducibility
14. Remaining unknowns
15. Final standing
```

The report must not contain:

```text
looks correct
should be branchless
likely optimized
appears safe
all good
production ready
mathematically proven
```

unless each phrase is replaced by a specific bounded claim and linked evidence.

---

# 32. Immediate purge conditions

An agent is removed from the active task when it:

* fabricates command output;
* reports a gate it did not run;
* hides a branch;
* copies the implementation into the oracle;
* uses meaningless arithmetic to satisfy a scanner;
* mutates state before validation;
* weakens a typed refusal into a fallback;
* disables a gate;
* excludes changed files from scanner jurisdiction;
* edits generated output manually;
* claims universal proof from random tests;
* reports source branchlessness as machine-code branchlessness.

The affected implementation is quarantined until independently reconstructed or repaired.

---

# 33. Final constitutional law

[
\boxed{
\text{No agent may trade structural truth for apparent progress.}
}
]

[
\boxed{
\text{No implementation may obtain standing from its own tests alone.}
}
]

[
\boxed{
\text{No adaptive mutation may occur without admitted evidence.}
}
]

[
\boxed{
\text{No authoritative runtime behavior may depend on hidden control flow.}
}
]

[
\boxed{
\text{The graph may be combinatorially maximal.}
}
]

[
\boxed{
\text{The machine must remain bounded, branchless, deterministic, and receipted.}
}
]

The next useful artifact is an accompanying `bcinr-cheat-scanner` rule specification that turns each constitutional prohibition into a named AST, generated-source, and object-code gate.

---

# 34. Hosted and cross-environment startup protocol

Hosted ChatGPT agents and agents operating across a GitHub connector, ephemeral
shell, or CI-observation boundary must read `CHATGPT-CLOUD-AGENTS.md` after this
constitution and before substantial repository work. That addendum governs
environment and evidence mechanics only. It cannot weaken any mathematical,
runtime, gate, ownership, or standing rule above.

Before beginning implementation, the agent must:

1. resolve the current default branch, requested target ref, and every applicable
   path-specific `AGENTS.md`;
2. inspect current source, manifests, tests, gate definitions, generated
   ownership, planning documents, and evidence artifacts relevant to the task;
3. classify the real operating mode as local checkout, GitHub connector only,
   hybrid, or CI observation;
4. distinguish observed source, executed commands, changed artifacts,
   independently verified facts, inferred conclusions, and blocked boundaries;
5. identify the highest-priority unfinished item that is actionable in the
   available mode;
6. verify that the item's assumptions still match current source and that the
   required constitutional workstream ownership is available;
7. name the first concrete contracts, authoritative roots, oracle surfaces,
   mutants, scanners, gates, or evidence artifacts involved;
8. begin useful work without asking the user to restate context already present
   in repository files.

The first substantive response must demonstrate this orientation rather than
merely summarize instruction documents. It must state the applicable instruction
hierarchy, real execution boundaries, and evidence required before the selected
item can receive its requested standing. It must begin the work in the same
response.

A blocked highest-ranked item is a truthful result. When another item is
independently actionable, select the next lawful checkpoint and record why the
higher item remains blocked. Never manufacture an execution path or standing to
avoid a blocker.

Do not rely on remembered repository details. Do not claim a command, gate,
proof, mutation kill, object-code property, receipt, replay, or CI boundary
passed unless it was actually executed or inspected for the exact commit and
exact command. Connector fetches are not `cat`, connector searches are not
`grep`, connector commits are not local working-tree edits, and remote metadata
is not `git status`.

Stay within the selected ticket's scope and exclusive write ownership. Never
hand-edit generated output. Use a dedicated branch, run the narrowest relevant
validation available, open a draft pull request when permitted, and do not merge
without explicit authorization.
