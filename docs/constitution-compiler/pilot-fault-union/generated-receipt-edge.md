# Generated receipt-DAG edge — pilot illustration

**Status:** ILLUSTRATIVE. Generated from ConstitutionIR claim `cmca.numeric.fault-join-semilattice`
(see `claim.yaml`), projected into the receipt-node shape defined in
`docs/constitution-compiler/03_RECEIPT_DAG_SCHEMA.md` §2-3. Not connected to any real receipt
emission, and no digest below is real BLAKE3 output — every digest is a fabricated placeholder,
exactly per that schema document's own §5 convention.

## Which node type this claim maps to

Per `03_RECEIPT_DAG_SCHEMA.md` §3's six-node vocabulary, a claim being "verified" corresponds to
the `verification` node type (generalizing the artifact contract's `validate` step and
`authority-and-c3.md`'s `AcceptedCertificate`/`AcceptedEnvelopeReceipt` shapes). This claim's
verification event depends on a prior `generation` node representing the property test having
been run (analogous to the artifact contract's `generate` step, repurposed here as "the property
test and mutant-kill check were executed").

## Worked receipt node — "this claim was verified"

```json
{
  "event": "verification",
  "inputs": {
    "claim_id": "cmca.numeric.fault-join-semilattice",
    "claim_digest": "blake3:c1a1a1a1c1a1a1a1c1a1a1a1c1a1a1a1c1a1a1a1c1a1a1a1c1a1a1a1c1a1a1a1",
    "property_test_receipt": "blake3:p2b2b2b2p2b2b2b2p2b2b2b2p2b2b2b2p2b2b2b2p2b2b2b2p2b2b2b2p2b2b2b2",
    "mutant_kill_receipt": "blake3:m3c3c3c3m3c3c3c3m3c3c3c3m3c3c3c3m3c3c3c3m3c3c3c3m3c3c3c3m3c3c3c3"
  },
  "outputs": {
    "verification_outcome_digest": "blake3:v4d4d4d4v4d4d4d4v4d4d4d4v4d4d4d4v4d4d4d4v4d4d4d4v4d4d4d4v4d4d4d4",
    "outcome": "pass",
    "evidence_classes_satisfied": ["property_test", "mutant_kill"]
  },
  "prev": "blake3:g0e0e0e0g0e0e0e0g0e0e0e0g0e0e0e0g0e0e0e0g0e0e0e0g0e0e0e0g0e0e0e0"
}
```

`R_verify = Hash("verification", inputs, outputs, prev)`

*(illustrative value, not computed by running BLAKE3 over anything)*:

```
blake3:5555eeee66667777ffff8888999900001111aaaa22223333bbbb4444cccc55
```

## Predecessor fan-in (per §2.2 of the receipt-DAG schema)

This node has **two** logical predecessors bound into `inputs` rather than a single `prev`
chain-link, because both `evidence_classes_required` entries in `claim.yaml`
(`property_test`, `mutant_kill`) must independently produce a passing receipt before the claim's
`verification` node can assert `outcome: pass` — this mirrors the schema's §5.1 fan-in extension
(`package` depending on both `generation` and `verification`), applied one level down at the
per-claim granularity rather than the per-release granularity the schema's own worked example
uses.

Using the schema's fan-in form:

```
R_verify = Hash("verification", sorted({R_property_test, R_mutant_kill}), outputs)
```

with `R_property_test` representing a receipt for `generated-property-test.rs`'s test run (not
itself modeled as a separate node in this pilot, for brevity) and `R_mutant_kill` representing a
receipt for `generated-mutant.md`'s kill confirmation (likewise not separately modeled here).

## Drift / friction note

The receipt-DAG schema's §3 node-type table names `verification`'s typical predecessor as
`consumption` (singular), not a fan-in of two independent evidence-class receipts. Modeling
*this* claim's verification correctly (both `property_test` AND `mutant_kill` must pass —
per `numeric-hot-path.md`'s own "Required Evidence Class" section: "both of the following, not
either alone") required reaching for the schema's §2.2 fan-in extension, which the schema
document itself flags as designed for release-level composition (e.g. `package` node), not
explicitly for per-claim, per-evidence-class composition. It works — the fan-in math is generic
— but this pilot had to make that reuse decision itself; the schema document does not name
"one claim's evidence classes fan in to one verification node" as a first-class use case.

## Nonclaims

No BLAKE3 hashing was performed to produce any digest above. This receipt edge does not
correspond to any real property test run or mutant kill result — `generated-property-test.rs`
in this directory was not executed against the real crate (see that file's own header), and
`generated-mutant.md`'s mutant was not authored or run against the real crate. This file is not
wired into, and does not affect, any real receipt chain in `bcinr-powl-receipt` or elsewhere.
