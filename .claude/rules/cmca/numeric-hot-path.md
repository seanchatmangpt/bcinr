---
paths:
  - "crates/bcinr-cmca/src/fixed.rs"
  - "crates/bcinr-cmca/src/allocator.rs"
---

# Numeric Hot-Path Invariants

This rule governs the numeric core of bcinr-cmca: fixed-point/checked arithmetic and the
allocator's budget accounting. It states timeless mathematical invariants for any code touching
these files. It is not a status report and asserts nothing about current compliance.

## Invariant 1 — Fault accumulation is a join-semilattice

The set of faults produced along a computation path forms a join-semilattice under union, with
the empty fault set as the zero (identity) element. Sequential composition of two fault-bearing
steps must union their fault sets: `faults(a ; b) = faults(a) ∪ faults(b)`. Composition must
never collapse to "keep only the first fault seen" or "keep only the last" — both are
short-circuits that lose information the lattice guarantees is preserved.

**Falsifier:** Construct two steps, each independently known to raise a distinct, non-overlapping
fault, compose them, and inspect the resulting fault set. A violation is observed if the composed
result's fault set is a strict subset of the union (e.g., only the first step's fault, or only
the last).

## Invariant 2 — Masked selection distributes over the fault-bearing pair

For a branchless/masked select over two `(value, fault)` alternatives, selection must distribute
over the pair as a whole:

```
select(m, (v_a, f_a), (v_b, f_b)) = (select(m, v_a, v_b), select(m, f_a, f_b))
```

Two distinct violations are in scope: (a) the fault of the *unselected* alternative leaking into
the result ("contamination"), and (b) the fault of the *selected* alternative being dropped
because the result's value is re-derived through a fresh "OK"/no-fault constructor after the
value is chosen ("silent erasure"). Both are prohibited regardless of whether the selected value
itself is numerically correct.

**Falsifier:** Construct alternative A with a faulted value and a specific fault tag, and
alternative B with a clean value and no fault. Select A under one mask setting and B under the
complement. Observe the fault field of each result: contamination is observed if selecting B
yields A's fault tag; erasure is observed if selecting A yields no fault tag.

## Invariant 3 — Canonical mask has a two-point publicly constructible image

For a canonical mask type of a given bit width, the set of values publicly constructible through
its safe API must be exactly `{0, all-ones}` for that width — no third value may be safely
reachable. This is what makes "masked select" a total, branchless boolean selector rather than an
arbitrary-bitpattern operation.

**Falsifier:** Enumerate or randomly sample the safe public constructors and any safe
transformations of the mask type, and check every producible value against `{0, all-ones}`. A
violation is observed the first time a safely constructed instance holds any other bit pattern.

## Invariant 4 — Exact-budget projections conserve the total

Any projection that floors or reserves a bounded discrete budget across a partition of items (a
"give each item its floor/reserved share") must conserve the total exactly: the sum of the parts
must equal the whole unit, with no unaccounted remainder and no manufactured excess. A per-item
rounded share that merely approximates the total is not sufficient — the parts must sum exactly,
which in general requires an explicit remainder-distribution step, not independent per-item
rounding.

**Falsifier:** Pick a budget and a partition size such that the budget does not divide evenly
across the partition. Compute the projected share for every item and sum them. A violation is
observed if the sum differs from the original budget by any nonzero amount, in either direction.

## Invariant 5 — Rejected authoritative operations leave state byte-for-byte unchanged

When an authoritative operation is rejected (refused, fails a precondition, fails admission), any
persistent state it would have mutated must be left byte-for-byte identical to its pre-attempt
value. "Logically equivalent" state (same observable value through some accessor, but a different
byte-level representation, generation counter, or padding) does not satisfy this invariant — the
requirement is at the representation level, not merely the accessor level.

**Falsifier:** Snapshot the raw bytes of the persistent state before attempting an operation
constructed to be rejected. Attempt the operation. Compare the raw byte snapshot to the
post-attempt bytes. A violation is observed on any byte-level difference, even one invisible
through the normal accessor API.

## Invariant 6 — The authoritative root is total

The authoritative root function of a bounded numeric computation must be total: for every input
in the admitted domain it must return a value together with a fault descriptor, on every call.
It must never be partial by early-returning, panicking, or using `Result`-as-control-flow at the
boundary a caller depends on to obtain a value — a caller must always be able to obtain a value
from the root call, with any anomaly represented in the fault descriptor rather than in the
absence of a return.

**Falsifier:** Enumerate or fuzz the admitted input domain, including its documented boundary and
edge cases, and call the authoritative root for each. A violation is observed the first time a
call fails to produce a `(value, fault)` pair — e.g., it panics, aborts, or returns early without
a value where a fault descriptor was the correct channel for the anomaly.

## Required Evidence Class

For each invariant, satisfying evidence consists of both of the following, not either alone:

1. A property test executed over the admitted input domain (not a fixed example set), driven by
   an independent, non-production-derived oracle — the oracle must be defined independently of
   the implementation under test, not derived by reading the implementation's own logic back out.
2. Comparison of the property test's result against that independent oracle, with any
   discrepancy treated as a violation of the corresponding invariant above.

Evidence that only exercises fixed examples, or that derives its oracle from the same code path
being tested, does not satisfy this requirement.

## Standing Consequence

A demonstrated violation of any invariant in this file blocks the numeric release gate for
bcinr-cmca. The gate does not reopen until a fix is evidenced per the required evidence class
above and recorded in the release ledger.

## Nonclaims

This rule makes no claim about whether the current implementation in `fixed.rs` or
`allocator.rs` satisfies any invariant above. That determination, any file:line evidence, and its
verification status are the exclusive domain of the release ledger — never of this rule. This
rule also does not itself constitute evidence; it only defines what evidence would be dispositive.

## See Also

- `/Users/sac/bcinr/.claude/agents/hoare-oracle.md` — formal-verification authority
- `/Users/sac/bcinr/.claude/agents/turing-machine.md` — totality/termination authority
- `/Users/sac/bcinr/.claude/agents/armstrong-fault.md` — fault-model authority
- `/Users/sac/bcinr/.claude/agents/von-neumann-bypass.md` — state/persistence authority
- `/Users/sac/bcinr/AGENTS.md` — constitution
