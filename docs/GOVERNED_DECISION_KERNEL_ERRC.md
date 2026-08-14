# Governed Decision Kernel — ERRC Grid

**Pattern under analysis:** `bcinr-cmca`'s deterministic fixed-point multi-criteria allocator +
typed refusal + `allocation_receipt.rs` audit-trail receipt + `cmca_rank_cli` general ranking
entry point + a calibrated DSPy/gpt-oss-20b secondary plausibility layer, now wired as a real
second consumer into `autofde-lab`'s `match_solvers(ranked=True)`.

**Companion doc:** `docs/GOVERNED_DECISION_KERNEL.md` (this session, same repo) — describes the
mechanism; this document evaluates it against real industry comparison points using the
Eliminate-Reduce-Raise-Create (ERRC) framework from Kim & Mauborgne's Blue Ocean Strategy.

**Relation to the user's existing ERRC work:** the user already has a full ERRC/Blue-Ocean
analysis in this ecosystem — `/Users/sac/gymact/docs/2026-08-12-chatman-ecosystem-errc-8020-1000x.md`
("The Chatman Ecosystem, read through ERRC / 80-20 / 1000x"). That doc's central finding is a
mass-vs-authority split: `autofde-lab` and `praxis` carry ~75% of the portfolio's code mass
while their own doctrine denies themselves production-actuation authority, and the doc names the
missing fourth stage (`autofde` — actual production authority, receipts, replay) as the "Raise"
gap. This grid is consistent with and extends that framing directly: `bcinr-cmca` wired into
`autofde-lab`'s `match_solvers(ranked=True)` is a small, concrete instance of exactly the
authority-bearing, receipted decision machinery the ecosystem doc says the portfolio is missing —
not the full `autofde` stage, but a real decision kernel with a typed refusal gate and an audit
receipt now actually consuming production input inside `autofde-lab`, rather than more
engineering mass added to a repo that still denies itself actuation authority. Where the
ecosystem doc flagged its own headline Create claim (`wasm4pm` OCEL wiring) as later found
**overstated** on independent check, this grid follows the same discipline below: every Raise/
Create item is checked against where it is actually weaker than a named real alternative, not
just where it wins.

---

## Eliminate

What this pattern removes that industry-standard LLM-decides or black-box-heuristic approaches
assume is necessary.

- **The LLM as primary decision-maker.** Per the LLM-judge/routing survey, the two most-adopted
  industry shapes both put an LLM (or an LLM-trained classifier fed by LLM-judged preference
  data) in the primary decision path: OpenAI's GPT-5 router is "a trained classifier +
  reinforcement signal," and DeepEval's G-Eval is "LLM-as-judge with chain-of-thought,"
  explicitly flagged by DeepEval's own docs as non-deterministic. `bcinr-cmca` eliminates this —
  the allocation decision is a deterministic fixed-point loop; gpt-oss-20b never picks the
  allocation, it only flags implausible outputs after the fact (the survey's own words: "an LLM
  or LLM-trained classifier is the primary mechanism" is the pattern being eliminated, not
  matched with a friendlier LLM).

- **The variance DeepEval names as its own known limitation.** G-Eval's non-determinism is not
  a hypothetical risk — DeepEval's docs "explicitly flag this as a known limitation," and their
  own writing on judge drift describes teams "burned because the signal stopped meaning what it
  used to the day the judge changed, and nobody had the audit trace to prove which day that
  was." A deterministic fixed-point core structurally cannot drift run-to-run on the same input
  the way a judge model can.

- **Silent best-effort degradation on malformed input.** The MCDA/scheduler survey found that
  three of four real allocation systems (Cluster Autoscaler, AHP/TOPSIS/SAW, ML multi-objective
  re-ranking) have **no formal typed refusal** — malformed input either gets filtered upstream
  informally or the ranker "degrades by scoring with defaults/imputation (best-effort), not by
  refusing." `bcinr-cmca`'s typed refusal (including the cyclic-parent check surfaced in
  `allocation_receipt.rs`) eliminates the best-effort-degrade default those three systems accept.

## Reduce

What this pattern deliberately does less of, compared to the more ambitious real alternatives
found in the surveys.

- **Cryptographic guarantee strength, relative to true verifiable computation.** Per the
  receipt/verifiable-computation survey, `allocation_receipt.rs`'s mix64 checksum is explicitly
  weaker than every cryptographic-hash-binding or proof system surveyed: below Certificate
  Transparency and SLSA (both SHA-256/signature-based), and far below zk-SNARKs ("full
  cryptographic proof of correct execution... verification uses only the public verification
  key and proof, no trust in the prover"). The survey states this directly: "mix64... gives
  error/mismatch detection (like a CRC)... not against a deliberate adversary constructing a
  colliding value, since mix64 has no claimed preimage/collision resistance." It is also
  explicitly weaker than **this codebase's own sibling design**, `bcinr-powl`'s
  `OcelCausalReceipt`, which is BLAKE3-based and would sit alongside SLSA/CT on the
  cryptographic-hash-binding tier. This is a real, named tradeoff, not a caveat buried
  elsewhere: the receipt buys recompute-and-compare auditability, not adversarial tamper
  resistance.

- **Generality of the allocator, relative to a fully general N-criteria/K-measure/Q-lens
  system.** The MCDA survey's K8s scheduler comparison shows production schedulers commonly
  blend 5-8 simultaneous scorers with per-plugin static weights; `bcinr-cmca`'s calibrated
  envelope (per the LLM-secondary-layer's own measured bound) is validated reliable only "above
  ~0.01 share-shift" — a real citable measurement from this session, not an invented industry
  benchmark. The pattern reduces scope to a calibrated operating envelope rather than claiming
  unconditional generality across all criteria magnitudes.

- **Weight-adaptation ambition, relative to production multi-objective re-ranking.** The MCDA
  survey names ML multi-objective re-ranking (DWA, FAMO, bandit arm selection) as having "the
  closest real precedent to MWU-style adaptation" with genuine **outcome-feedback-adaptive**
  weighting learned online. `bcinr-cmca`'s MWU/KL-guard/dwell-time hysteresis loop adapts weights
  within a run's fixed-point convergence, not from realized outcome feedback across many
  deployed decisions the way DWA/FAMO do — a narrower, single-decision adaptation scope rather
  than the cross-run learning ambition of production re-ranking systems.

## Raise

What this pattern pushes above what the real comparison points typically offer.

- **Refusal formality, relative to the two systems with any refusal at all.** Per the MCDA
  survey summary table, only K8s scheduler scoring and Cluster Autoscaler expanders have a
  formal admission/refusal gate among the four systems studied, and even those are "a separate
  Filter phase" or "feasibility filter" bolted alongside the scorer, not a typed refusal
  co-located with the allocation and receipt logic the way `bcinr-cmca`'s refusal and
  `allocation_receipt.rs`'s cyclic-parent check are. AHP/TOPSIS/SAW and ML re-ranking have no
  formal refusal at all.

- **Receipting, relative to every system in three of the four surveys.** The receipt survey's
  strongest evidence point is DeepEval's own team naming "the absence of an audit trace as the
  actual production failure they hit" — described in the LLM-judge survey as "the strongest
  evidence found that this is an unaddressed gap in the current standard shape, not a solved
  problem elsewhere." The MCDA survey's summary table confirms: zero of the four MCDA/scheduler
  systems studied have any cryptographic receipt — all stop at "a log line or event record."
  `bcinr-cmca` raises past a bare log to a recompute-and-compare seal/verify receipt, even at
  the reduced (mix64, not BLAKE3) guarantee strength named above.

- **Structural position of the LLM relative to the deterministic core, matching the one
  precedent found that does this deliberately.** The LLM-judge survey names Guardrails AI as
  "the one pattern found that structurally matches 'deterministic core, LLM as secondary
  check'" — and even Guardrails AI logs to an MLflow-style trace, not a cryptographic receipt.
  `bcinr-cmca`'s DSPy/gpt-oss-20b layer matches Guardrails AI's structural placement (secondary
  plausibility check, not primary decision) while raising past it on the receipting axis named
  above.

## Create

What this pattern offers that, per the real surveys, has no clean existing precedent.

- **The specific four-way combination.** The MCDA survey states this explicitly in its own
  summary: "None of these four found a cryptographic/auditable receipt binding a specific
  decision to its inputs — every real system checked stops at a log line or event record. That,
  plus the K-measures × Q-lenses simultaneous blend with MWU/KL-guard/dwell-time hysteresis
  combined in one deterministic fixed-point loop, and a typed admission/refusal gate, appear to
  be the differentiating combination not found together in any single one of these four industry
  precedents (each system had at most 2 of the 4 differentiators)." The LLM-judge survey
  independently confirms the same gap from the routing/guardrails side: "None of the five found
  patterns combine all of: (a) deterministic core decision, (b) typed/structured refusal, (c)
  cryptographic (BLAKE3-style) receipt, and (d) a calibrated, magnitude-thresholded secondary
  LLM check." Two independent surveys converging on the same absence is the actual grounding for
  this Create claim, not a single survey's framing.

- **A calibrated, magnitude-thresholded secondary LLM layer as a named, measured envelope rather
  than an unquantified "LLM double-checks it."** The closest structural analogue, Redwood
  Research's AI Control trusted-monitor protocols, is "by design... always a secondary layer,"
  matching the structural placement — but the survey is explicit that "it monitors agent
  *behavior*, not a deterministic allocation decision, and it carries no receipting," and the
  monitor's judgment is "non-deterministic by construction, since the monitor is itself an LLM"
  with no stated reliability envelope. `bcinr-cmca`'s secondary layer instead reports a
  **measured** operating bound (0% false-positive rate; unreliable below ~0.01 share-shift) —
  a citable number from this session, not an industry claim. Say the honest limit plainly: this
  measured envelope is narrower than a full eval-harness approach (e.g., DeepEval's DAGMetric
  decision-tree, built precisely to give traceable scoring paths across a broad criteria space)
  would achieve — the calibration is real but scoped to this allocator's own output distribution,
  not validated as a general-purpose judge replacement.

- **Wiring into a second real consumer (`autofde-lab`'s `match_solvers(ranked=True)`) rather than
  remaining a standalone library demo.** None of the four surveys' industry systems were found
  bridging a deterministic-core-plus-receipt allocator into an external production ranking call
  path the way this session did; this is a "no counter-precedent found" claim in the sense the
  criticism-discipline rules require — an absence noted across four targeted searches, not a
  positive claim that no such integration exists anywhere in industry.

---

## Honest tradeoff summary (what this grid gives up, not just wins)

- The receipt is real but cryptographically **weaker** than this same codebase's own
  `OcelCausalReceipt` (BLAKE3) and weaker than CT/SLSA/zk-SNARKs — a CRC-grade integrity check,
  not an adversarial-tamper-resistant one.
- The allocator's validated reliability envelope is **narrower** than a full N-criteria
  production scheduler's blend (5-8 simultaneous scorers) and narrower than what a general
  eval-harness (DAGMetric-style decision tree) would cover.
- The MWU/hysteresis adaptation is **scoped to within-run convergence**, not the cross-run,
  outcome-feedback-adaptive weighting that production multi-objective re-ranking (DWA/FAMO/
  bandits) has demonstrably shipped.
- The "no existing precedent" Create claims rest on four targeted surveys, not an exhaustive
  industry census — following the same self-critical discipline the user's existing ecosystem
  ERRC doc modeled when its own headline Create claim was later checked and found overstated.

## See Also

- `docs/GOVERNED_DECISION_KERNEL.md` — mechanism description for the pattern this grid evaluates
- `/Users/sac/gymact/docs/2026-08-12-chatman-ecosystem-errc-8020-1000x.md` — the ecosystem-level
  ERRC/80-20/1000x analysis this grid extends
- `crates/bcinr-cmca/` — the deterministic fixed-point allocator and `allocation_receipt.rs`
- `crates/bcinr-powl/src/receipt.rs` — `OcelCausalReceipt`, the stronger BLAKE3-based sibling
  design referenced in the Reduce section
