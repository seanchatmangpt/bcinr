---
paths:
  - "crates/bcinr-cmca/src/fixed.rs"
  - "crates/bcinr-cmca/src/allocator.rs"
# GENERATED — pilot illustration only. No projector exists; this file was hand-authored to
# show what a projector, reading claim.yaml in this directory, would emit into a
# .claude/rules/cmca/*.md-shaped file. It is NOT wired to replace, and does not replace,
# the real /Users/sac/bcinr/.claude/rules/cmca/numeric-hot-path.md.
---

# Fault Accumulation Is a Join-Semilattice

<!-- GENERATED from ConstitutionIR claim id: cmca.numeric.fault-join-semilattice -->
<!-- Source: docs/constitution-compiler/pilot-fault-union/claim.yaml -->
<!-- This projection covers exactly one invariant. The real numeric-hot-path.md covers six. -->

This rule governs `fixed.rs` and `allocator.rs`. It states a timeless mathematical invariant;
it is not a status report and asserts nothing about current compliance.

## Invariant — Fault accumulation is a join-semilattice

The set of faults produced along a computation path forms a join-semilattice under union, with
the empty fault set as the zero (identity) element. Sequential composition of two fault-bearing
steps must union their fault sets: `faults(a ; b) = faults(a) ∪ faults(b)`. Composition must
never collapse to "keep only the first fault seen" or "keep only the last" — both are
short-circuits that lose information the lattice guarantees is preserved.

**Falsifier:** Construct two steps, each independently known to raise a distinct, non-overlapping
fault, compose them, and inspect the resulting fault set. A violation is observed if the composed
result's fault set is a strict subset of the union (e.g., only the first step's fault, or only
the last).

## Required Evidence Class

- `property_test` — a property test over the admitted input domain, oracle defined
  independently of the implementation.
- `mutant_kill` — the corresponding mutant (`generated-mutant.md`, this directory) must be
  demonstrably killed.

## Standing Consequence

A demonstrated violation blocks the numeric release gate for the scope above. The gate does not
reopen until a fix is evidenced per the required evidence class and recorded in the release
ledger.

## Nonclaims

This rule makes no claim about whether the current implementation in `fixed.rs` or
`allocator.rs` satisfies the invariant above. That determination is the exclusive domain of the
release ledger.

## Drift note (comparing this projection against the real rule file)

Compared against the real, hand-authored `/Users/sac/bcinr/.claude/rules/cmca/numeric-hot-path.md`:

- **Consistent for the one invariant this projection covers.** The invariant statement,
  falsifier, and Nonclaims wording above are near-verbatim reproductions of the real file's
  Invariant 1 section — no semantic drift found for this specific claim.
- **Coverage gap, not a contradiction.** The real rule file states six invariants (fault
  join-semilattice, masked-select distribution, canonical two-point mask image, exact-budget
  conservation, byte-invariant rejection, root totality). This projection, being generated from
  a single claim instance, covers only Invariant 1. A real projector would need six claim
  instances (one per invariant) feeding one rule-file template to reproduce the real document in
  full — this pilot deliberately did not author the other five as YAML, so this generated file
  is honestly a strict subset of the real one, not a full replacement.
- **"See Also" section omitted.** The real file links to `hoare-oracle.md`, `turing-machine.md`,
  `armstrong-fault.md`, `von-neumann-bypass.md`, and `AGENTS.md`. Reproducing that section from
  `claim.yaml` alone is possible only for the `owner`/`verifier` agents named in the claim
  (`cmca-numeric`, `hoare-oracle`) — the other three agent links in the real file are not
  derivable from this claim's fields at all, because they are cross-references relevant to the
  rule *file* as a whole (all six invariants), not to this one claim. This is a concrete
  friction point: a per-claim IR does not obviously determine a per-*file* See Also section
  without an additional aggregation step above the claim level.
