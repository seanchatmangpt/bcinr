# Generated ledger gate entry — pilot illustration

**Status:** ILLUSTRATIVE. Generated from ConstitutionIR claim `cmca.numeric.fault-join-semilattice`
(see `claim.yaml`), projected into the 9-tuple gate shape defined in
`docs/constitution-compiler/04_LEDGER_STATE_MACHINE.md` §1. This is NOT an entry in, and is not
appended to, the real `/Users/sac/bcinr/docs/cmca-rdf/V26_7_17_RELEASE_LEDGER.md`, which is being
independently advanced right now by a separate background workflow.

## Gate tuple projection

```
g = (id, standing, owner, verifier, dependencies, falsifier,
     repair_capability, verification_capability, evidence_requirements)
```

| Field | Value (projected from claim.yaml) |
|---|---|
| `id` | `cmca.numeric.fault-join-semilattice` (reused directly as the gate id — see Drift note below) |
| `standing` | *(not set by this projection — see Nonclaims. A real gate's `standing` is never derivable from a claim alone; it is recorded by verification, not by the IR.)* |
| `owner` | `cmca-numeric` (taken verbatim from claim.yaml's `owner` field) |
| `verifier` | `hoare-oracle` (taken verbatim from claim.yaml's `verifier` field) |
| `dependencies` | `[]` (taken verbatim from claim.yaml's `dependencies` field — this claim has none) |
| `falsifier` | "Construct two steps, each independently known to raise a distinct, non-overlapping fault, compose them, and inspect the resulting fault set. A violation is observed if the composed result's fault set is a strict subset of the union." (claim.yaml `falsifier_family[0]`, concatenated) |
| `repair_capability` | Fix the composition operator in `crates/bcinr-cmca/src/allocator.rs` (or `fixed.rs`) to perform set union rather than first/last-wins selection at the sequential-composition site named in `generated-mutant.md` |
| `verification_capability` | Run `generated-property-test.rs`'s two proptest cases (bound to the real production composition function, once such a binding exists) plus confirm `generated-mutant.md`'s mutant is killed |
| `evidence_requirements` | `[property_test result, mutant_kill result for fault-union-restore-first-error-sentinel]` — directly the two entries of claim.yaml's `evidence_classes_required` list |

## Worked example under the three transition laws (04_LEDGER_STATE_MACHINE.md §2)

Purely illustrative application of the three laws to this one gate — no dispatch mechanism
exists to actually execute any of this:

- **Law 1** (`Standing(g) = BLOCKED => Dispatch(Owner(g), RepairIntent(g))`): if this gate's
  standing were `BLOCKED` (e.g. because the real ledger's G2 entry, read read-only, currently
  cites a missing `NumericFaultSet`/`RefusalSet` type decision blocking this exact invariant),
  the law would dispatch `cmca-numeric` with a `RepairIntent` carrying this gate's `falsifier`
  and `repair_capability` fields as payload.
- **Law 2** (`ArtifactChanged(g) => Dispatch(Verifier(g), VerifyIntent(g))`): if
  `crates/bcinr-cmca/src/allocator.rs` changes on disk, the law would dispatch `hoare-oracle`
  with a `VerifyIntent` to re-run `verification_capability` above.
- **Law 3** (`ReceiptValid(g) => RecomputeStanding(g)`): if `hoare-oracle`'s re-run produces a
  receipt matching `generated-receipt-edge.md`'s shape with `outcome: pass`, the law would
  recompute this gate's standing from that receipt, not from `cmca-numeric`'s self-report.

## Drift note (comparing this projected gate against the real ledger's G2 entry)

The real ledger's G2 ("Numeric Law") gate, read read-only, is broader than this one claim: G2
covers the missing `NumericFaultSet`/`RefusalSet` type decision, the `from_bits`-fault-dropping
sites, the `const_eq_u32` signature reconciliation, AND this fault-union invariant, all under one
gate id (`G2`). This pilot's projected gate uses the claim's own dotted id
(`cmca.numeric.fault-join-semilattice`) as the gate id instead of `G2` — a real projector would
need an explicit many-claims-to-one-gate aggregation rule (which claims roll up into `G2` vs. get
their own gate) that neither `claim.yaml`'s schema nor `04_LEDGER_STATE_MACHINE.md`'s tuple shape
currently specifies. This is a friction point named in `SUMMARY.md`: the ledger's granularity
(one gate per named release concern) and the IR's granularity (one gate-candidate per atomic
claim) do not coincide 1:1, and nothing in either design document decides the mapping.

## Nonclaims

This file does not set, imply, or record any actual `standing` value for
`cmca.numeric.fault-join-semilattice` or for the real ledger's `G2`. No dispatch, repair, or
verification described above has been executed. The real ledger is unmodified by this file.
