# Standing Algebra and Evidence Discipline

## Law

Every claim about this repository carries exactly one standing drawn from a closed
vocabulary. The base algebra is defined in AGENTS.md section 28: `PROVEN`, `INVARIANT`,
`ALIVE`, `SOURCE_BRANCHLESS_PARTIAL`, `BRANCHLESS_ALIVE`, `REPORTED_ALIVE`, `PARTIAL_ALIVE`,
`UNKNOWN`, `REFUSED`, `BUILD_BROKEN`. This release adds one label to that algebra: `REPORTED`
— a claim made by an implementation agent about its own work, not yet mechanically reproduced
by an independent verifier.

A composite claim's standing is bounded above by the weakest standing among its load-bearing
dependencies. A claim depending on any `UNKNOWN`, `REFUSED`, `BUILD_BROKEN`, or `REPORTED`
component cannot itself be promoted past that component's standing, regardless of how strong
the claim's own local evidence is. Promotion of a dependency does not retroactively promote
claims that were recorded while it was weaker; each dependent claim must be re-evaluated and
re-recorded against the dependency's new standing.

Gap-closure protocol: any newly discovered in-scope defect must be (1) named as a discrete
claim, (2) assigned an owner and an independent verifier distinct from the owner, (3) given a
falsifier and the evidence class that would satisfy it, and (4) recorded as a standing entry.
A defect may never be absorbed into prose, folded into an unrelated claim, or silently renamed
to obscure that it is the same open gap.

## Falsifier

This rule is violated by any of the following observable behaviors:

- A claim is asserted at a standing (e.g. `ALIVE`, `BRANCHLESS_ALIVE`) higher than the weakest
  standing among the dependencies it actually rests on.
- A claim originating from an implementation agent's self-report is recorded or treated as
  anything other than `REPORTED` before an independent verifier has mechanically reproduced it.
- A defect is discovered during work and does not appear as a named, owned, falsifiable entry
  anywhere — it exists only as a sentence in a summary, a code comment, or a renamed ticket
  that erases its history.
- A dependency's standing changes and a claim that depended on it keeps its prior recorded
  standing without re-evaluation.

## Required Evidence Class

Promotion from `REPORTED` to any stronger standing requires an independent verifier — a party
or process distinct from the reporting agent — to mechanically reproduce the claimed result
(re-run the falsifier, re-derive the proof, re-execute the test) and record that reproduction
as a discrete, attributable event. A verifier's own attestation is itself subject to this same
algebra: it is not exempt from carrying a standing.

## Standing Consequence of Violation

Any artifact, report, or commit message that assigns a standing exceeding what this algebra
permits is itself `REFUSED` on inspection: the offending claim must be re-labeled to its
correct bound (or to `REPORTED`/`UNKNOWN` if no independent verification exists) before the
artifact may be treated as authoritative. A repeated pattern of unsupported promotion is a
defect in its own right and must go through the gap-closure protocol above.

## Nonclaims

This rule states the vocabulary, the bounding law, and the gap-closure protocol only. It
contains no current status, no file:line reference, and no version-scoped fact — all current
statuses, promotions, and open defects for this release live exclusively in the mutable
release ledger at `docs/cmca-rdf/V26_7_17_RELEASE_LEDGER.md`, which this rule points to by path
and never quotes. Agent-specific ownership of verification duties is defined in the sibling
agent definitions under `.claude/agents/`, not here. Enforcement mechanics (what gets checked,
at what cost, at which of the three hook levels) are defined in the sibling hook/enforcement
specification, not here.
