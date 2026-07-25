# Receipt DAG Schema — Constitutional Compiler v0 (Design Draft)

**Status:** exploratory design for a future post-v26.7.17 milestone. ILLUSTRATIVE / NOT-WIRED.
**Kind:** speculative schema proposal, not a specification of any currently-running system.

## 0. Scope and non-claims

This document generalizes the receipt shapes already documented in this workspace —
`CertificateReceipt` / `EnvelopeReceipt` / `OutcomeReceipt` (AGENTS.md §11, ReceiptSound law) and
`cmca_generation_receipt.json`'s four-step chain (`docs/cmca-rdf/CMCA_ARTIFACT_CONTRACT.md` §2.1)
— into one hash-chained DAG schema, for possible future use by a "Constitutional Compiler" that
projects release-constitution artifacts from a common IR.

Explicit non-claims:

- This schema is **not connected** to any currently-running receipt emission, the
  `bcinr-cmca` crate, the active `V26_7_17_RELEASE_LEDGER.md`, or any receipt file actually
  produced by this repository's build or release process.
- No digest, hash, or example value in this document is real. Every digest shown below is
  fabricated for illustration and is labeled as such.
- This document does not modify, and was not used to modify, anything under
  `crates/bcinr-cmca/**`, `.claude/agents/cmca-*.md`, `docs/cmca-rdf/**`, or
  `crates/bcinr-cmca/MUTANT_KILL_MATRIX.md`. It only reads those paths as reference.
- Adopting this schema for a real release would require a separate, later design and
  implementation decision; this document is an input to that decision, not the decision itself.

## 1. Why generalize

Three receipt shapes already exist in this workspace, independently authored:

| Existing shape | Where documented | Chain discipline |
|---|---|---|
| `CertificateReceipt` / `EnvelopeReceipt` / `OutcomeReceipt` | AGENTS.md §11 (ReceiptSound law) | conjunctive gate — all three must be accepted before adaptive mutation; no explicit hash-chain field is documented in AGENTS.md itself |
| `cmca_generation_receipt.json` | `docs/cmca-rdf/CMCA_ARTIFACT_CONTRACT.md` §2.1 | linear 4-step chain: `admit_graph → validate → generate → emit_artifact`, each step's `prev` pointing at the previous step's `digest`, rolled up into `final_digest` |
| POWL execution receipts | `bcinr-powl-receipt` crate (BLAKE3-chained, per `bcinr-mcp` `receipt_inspect` tool) | not read in detail for this document; noted here only as a third existing precedent for BLAKE3-chained receipts in this workspace |

All three are hash-chained event logs at heart, differing only in step vocabulary and in
whether the chain is purely linear or has fan-in. The schema below is one generalization that
both existing shapes can be read as specializations of — a claim about *shape compatibility*,
not a claim that either existing shape has been migrated to it.

## 2. Core recurrence

### 2.1 Linear case

For a chain of receipt nodes `0, 1, ..., n`, each node's receipt digest is:

```
R_0 = Hash(event_0, inputs_0, outputs_0, GENESIS)
R_n = Hash(event_n, inputs_n, outputs_n, R_{n-1})   for n >= 1
```

- `event_n` — a short, stable string naming the step kind (e.g. `"admission"`, `"generation"`).
- `inputs_n` — canonical-JSON-encoded (JCS-style, per `CMCA_ARTIFACT_CONTRACT.md` §3.2) digest
  references to whatever this step consumed. Not raw bytes — digests of raw bytes, so the
  receipt stays small regardless of artifact size.
- `outputs_n` — canonical-JSON-encoded digest references to whatever this step produced.
- `R_{n-1}` — the previous node's own receipt digest, or a fixed `GENESIS` sentinel
  (e.g. the all-zero BLAKE3 digest) for the first node in a chain.
- `Hash` — BLAKE3, matching the algorithm already fixed by `CMCA_ARTIFACT_CONTRACT.md` §3.2 for
  the existing six-digest list. This document does not introduce a second hash algorithm.

This is exactly the shape `cmca_generation_receipt.json` already uses (§2.1 of the artifact
contract), restated with `inputs_n` / `outputs_n` split out explicitly rather than left implicit
in "that step's output bytes."

### 2.2 Branching case (fan-in)

A node may depend on more than one predecessor receipt — for example, a package step that
depends on both a generation receipt and an independently-produced verification receipt. For a
node with `k` predecessor receipts `R_{p1}, ..., R_{pk}`:

```
R_n = Hash(event_n, {R_p1, ..., R_pk}, outputs_n)
```

Notes on this extension:

- `{R_p1, ..., R_pk}` is encoded as a **sorted** list (byte-wise sort on the hex digest string,
  per the existing "object keys sorted lexicographically" canonicalization rule in
  `CMCA_ARTIFACT_CONTRACT.md` §3.2) so that `Hash` is insensitive to the order predecessors were
  discovered or listed in — the DAG structure carries the meaning, not incidental list order.
- `inputs_n` is dropped from the branching form's argument list because the predecessor receipt
  set already transitively commits to every predecessor's own `inputs`/`outputs` — re-listing
  them would be redundant, not additional binding strength. A node MAY still carry its own
  direct `inputs_n` (e.g. a config value read fresh at this step, not derived from any
  predecessor) — when it does, the full form is
  `R_n = Hash(event_n, {R_p1, ..., R_pk}, inputs_n, outputs_n)`, with the two-argument form
  above being the special case `inputs_n = ∅`.
- The linear case (§2.1) is the special case `k = 1`, with `R_{n-1}` playing the role of the
  single-element predecessor set `{R_p1}`.
- This structure is a Merkle DAG, not a Merkle tree: multiple downstream nodes may each cite the
  same upstream receipt as a predecessor (e.g. both a `package` step and a separate `audit`
  step citing the same `verification` receipt) without that upstream receipt being consumed or
  duplicated.

## 3. Receipt-node types for a CMCA-style release

The following six node types are proposed as the vocabulary for `event_n` in a CMCA-style
release chain. Each generalizes an existing, already-documented step:

| Node type | Generalizes | Typical inputs | Typical outputs | Typical predecessors |
|---|---|---|---|---|
| `admission` | `admit_graph` step (artifact contract §2.1); `AdmittedControlState` (AGENTS.md §11) | raw RDF graph digest, admission policy digest | admitted-graph digest, per-shape pass/fail record | none (chain root) or a prior release's `publish-dry-run` |
| `generation` | `generate` step (artifact contract §2.1) | admission receipt, generator source digest | `cmca_generated.rs` payload digest, generation manifest digest | `admission` |
| `consumption` | not separately named in existing docs; the act of a downstream crate reading a generated artifact and binding its own build to it (`VerifyGeneratedProfile`, artifact contract §4) | generation receipt, consuming crate's build config digest | consumer-side verification-passed digest | `generation` |
| `verification` | `validate` step (artifact contract §2.1); `AcceptedCertificate` / `AcceptedEnvelopeReceipt` (AGENTS.md §11) | consumption receipt, verification policy digest (e.g. mutant-kill matrix pass criteria) | verification-outcome digest (pass/fail plus per-binding detail, per Invariant 3 in `authority-and-c3.md`) | `consumption` |
| `package` | `emit_artifact` step (artifact contract §2.1), extended to fan-in | generation receipt AND verification receipt (fan-in — this is the canonical two-predecessor example) | packaged release-artifact digest | `generation`, `verification` |
| `publish-dry-run` | `AcceptedOutcomeReceipt` (AGENTS.md §11), restricted to a non-actuating dry run | package receipt, dry-run policy digest | dry-run outcome digest (never an actual publish action) | `package` |

Two points worth flagging explicitly, both UNVERIFIED against any real implementation:

- Whether `consumption` should be its own node type or folded into `verification` is a design
  choice this document does not resolve; it is listed separately here because AGENTS.md §11's
  four-part conjunction (`AdmittedControlState`, `AcceptedCertificate`, `AcceptedEnvelopeReceipt`,
  `AcceptedOutcomeReceipt`) plus `CertifiedLearningMode` suggests at least one more granularity
  step than the artifact contract's four-step chain names.
- `publish-dry-run` is named to make explicit that this schema, as drafted, never reaches an
  actual publish/actuation node — consistent with the Nonclaim in `authority-and-c3.md` about
  actuation surfaces being a structural finding, not an assumption. A real `publish` node, if
  ever added, is out of scope for this document.

## 4. Composing into one ReleaseArtifact

```
ReleaseArtifact = (package, receipt_root, standing_projection)
```

- `package` — the actual release payload bytes (or a manifest of file digests referencing them),
  i.e. what `emit_artifact` / `package` produced.
- `receipt_root` — the receipt digest of the DAG's terminal node (here, `publish-dry-run`'s
  `R_n`), which transitively commits to every ancestor node's `event`, `inputs`, `outputs`, and
  predecessor set, all the way back to the chain's `admission` root(s).
- `standing_projection` — a derived, human/tool-readable summary (e.g. ALIVE / PARTIAL /
  BLOCKED / UNSUPPORTED per node, mirroring the vocabulary in
  `~/.claude/rules/no-overclaiming-rust.md`) computed *from* `receipt_root`'s DAG, not stored
  independently of it — so that a standing claim can always be re-derived and checked against
  the hash chain rather than trusted as free-floating prose. This is the specific mechanism by
  which this design proposes to address the "constitution hand-authored in five places"
  problem named in the milestone's framing: `standing_projection` would be *projected from* the
  DAG, not separately hand-maintained.

`receipt_root` alone is sufficient to detect any tampering with `package`: recomputing the
`package` node's `outputs_n` digest and comparing it against the value bound inside the chain
that led to `receipt_root` will mismatch if `package`'s bytes changed after the fact.

## 5. Worked example (4-node linear DAG, fabricated digests)

**All digests below are illustrative placeholders, not real BLAKE3 output. They are shortened
hex strings chosen for readability, not computed by running BLAKE3 over anything.**

Chain: `admission -> generation -> consumption -> verification` (a linear 4-node slice; no
fan-in in this particular worked example — the fan-in case is shown separately in §5.1).

### Node 0 — `admission`

```json
{
  "event": "admission",
  "inputs": { "rdf_digest": "blake3:aaaa1111...aaaa" },
  "outputs": { "admitted_graph_digest": "blake3:bbbb2222...bbbb" },
  "prev": "blake3:00000000000000000000000000000000000000000000000000000000000000"
}
```

`R_0 = Hash("admission", inputs_0, outputs_0, GENESIS)`
`R_0 = blake3:1111aaaa22223333bbbb4444555566667777cccc8888999900001111aaaa22`
*(illustrative value — not computed)*

### Node 1 — `generation`

```json
{
  "event": "generation",
  "inputs": { "admission_receipt": "R_0", "generator_digest": "blake3:cccc3333...cccc" },
  "outputs": { "generated_payload_digest": "blake3:dddd4444...dddd" },
  "prev": "R_0"
}
```

`R_1 = Hash("generation", inputs_1, outputs_1, R_0)`
`R_1 = blake3:2222bbbb33334444cccc5555666677778888dddd9999000011112222bbbb33`
*(illustrative value — not computed)*

### Node 2 — `consumption`

```json
{
  "event": "consumption",
  "inputs": { "generation_receipt": "R_1", "consumer_build_digest": "blake3:eeee5555...eeee" },
  "outputs": { "consumer_verified_digest": "blake3:ffff6666...ffff" },
  "prev": "R_1"
}
```

`R_2 = Hash("consumption", inputs_2, outputs_2, R_1)`
`R_2 = blake3:3333cccc44445555dddd6666777788889999eeee0000111122223333cccc44`
*(illustrative value — not computed)*

### Node 3 — `verification`

```json
{
  "event": "verification",
  "inputs": { "consumption_receipt": "R_2", "mutant_kill_policy_digest": "blake3:00007777...0000" },
  "outputs": { "verification_outcome_digest": "blake3:11118888...1111", "outcome": "pass" },
  "prev": "R_2"
}
```

`R_3 = Hash("verification", inputs_3, outputs_3, R_2)`
`R_3 = blake3:4444dddd55556666eeee7777888899990000ffff1111222233334444dddd55`
*(illustrative value — not computed; this would serve as `receipt_root` if the chain terminated here)*

Each `R_n` above was written as a plausible-looking hex string purely for legibility of the
worked example; **no BLAKE3 hashing was actually performed to produce any of them**, and none
should be compared against, or substituted into, any real receipt file.

### 5.1 Fan-in extension of the same example

If a fifth node `package` depended on both `R_1` (`generation`) and `R_3` (`verification`)
directly — skipping `consumption` as a predecessor of `package` in this variant — its digest
would be:

```
R_4 = Hash("package", sorted({R_1, R_3}), outputs_4)
```

with `sorted({R_1, R_3})` meaning the two hex strings `R_1` and `R_3` byte-sorted before hashing,
so that a hypothetical alternative construction order `{R_3, R_1}` produces the identical `R_4`.

## 6. Open questions for a later design pass (explicitly deferred)

- Whether `standing_projection` (§4) should itself be a receipt node in the DAG (making the
  full `ReleaseArtifact` triple collapse into a single receipt-rooted structure) or a derived
  view computed on demand — not resolved here.
- Whether the six-digest canonical list in `CMCA_ARTIFACT_CONTRACT.md` §3.1 should be
  subsumed as one `generation` node's `outputs`, or kept as a parallel, separately-versioned
  manifest referenced *by* digest from the `generation` node — not resolved here.
- Whether `bcinr-powl-receipt`'s existing BLAKE3-chained POWL execution receipts (read
  read-only via the `receipt_inspect` MCP tool for this document, not otherwise inspected in
  detail) already implement a compatible fan-in shape, making them a nearer-term proof of
  concept for §2.2 — not investigated in this task.

## See Also

- `AGENTS.md` §11 (ReceiptSound law) — the four-part conjunctive receipt-acceptance gate this
  schema's `verification` / `publish-dry-run` nodes generalize.
- `docs/cmca-rdf/CMCA_ARTIFACT_CONTRACT.md` §2.1, §3 — the linear four-step receipt chain and
  six-digest canonicalization rules this schema's linear case and JCS encoding reuse.
- `.claude/rules/cmca/authority-and-c3.md` — Invariant 3 (seal must bind every enumerated
  identity) and the actuation-surface Nonclaim, both referenced in §3 above.
