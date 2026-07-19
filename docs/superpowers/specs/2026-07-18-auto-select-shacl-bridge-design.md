# Auto Select: SHACL-to-Selection Bridge (80/20 production slice)

Status: approved for implementation planning | Date: 2026-07-18

## Context

A design (`docs/auto_select_semantic_projection.md`) proposes a pipeline — RDF semantics →
SHACL eligibility → numeric measure vector → CMCA-style selection → picked tool — for choosing
among candidate tools (e.g. `SPARQLQueryTool`, `RulesEngineTool`, `VectorSearchTool`) based on
admitted facts about a request and each candidate.

A corpus survey (this session) found the pipeline was never actually wired, and surfaced a
governance problem alongside it:

- **Two incompatible real implementations exist**, not one orphaned pipeline:
  - `bcinr_logic::autonomic::auto_select` (`crates/bcinr-logic/src/autonomic/auto_select.rs`) +
    `bcinr_powl::auto_select_bridge` (`crates/bcinr-powl/src/auto_select_bridge.rs`) — minimal
    (`candidates: [u32; 8]`, `valid_mask: u8`, branchless argmax), but **real and tested**:
    proptest + hostile-mutant coverage on both the selection primitive and the POWL bridge.
  - `/Users/sac/mfw/mfw-auto-select` — a richer shape (`ToolCandidate` with 7 named
    coordinates, `q_lens`, mass²) matching the design doc's struct names, plus POWL/epoch/
    chaos/trace/causal-buffer integration, but its own bridge functions
    (`powl_bridge_select`, `powl_admit_selection`, `powl_ingest_receipt`, `mfw_apply_receipt`)
    have **zero tests**, two of its feature docs contradict their own real source
    (`trace.rs`, `chaos.rs`), and its object-audit docs
    (`mfw_auto_select_object_audit.md`, `logic_auto_select_object_audit.md`,
    `powl_auto_select_bridge_object_audit.md`) declare **"PhD-Verified and ALIVE standing.
    Merging authorized"** self-certified by the same role with no independent verifier named —
    one of them cites a disassembly/cheat-scan run with no corresponding artifact on disk. This
    is a direct violation of this repo's `.claude/rules/30-authority-separation.md`
    no-self-certification clause. **Not addressed by this spec** — named here so it isn't
    silently treated as settled; a separate pass should retract or re-derive those claims.
- **No real connection to SHACL/RDF admission exists anywhere.** The one seam
  (`mfw-auto-select::translate_shacl_eligibility`) is a bit-packer that assumes eligibility was
  already decided elsewhere — nothing calls it, and nothing in `mfw-meaning` (the crate that
  actually does SHACL admission, via `praxis-graphlaw`) references any of this.
- **This is not the same gap as the CMCA release ledger's C4_projection** (`docs/cmca-rdf/
  V26_7_17_RELEASE_LEDGER.md` migration step 3, "port ontology/generator into mfw"). That gap —
  generating the `Gamma_CMCA` numeric-allocator profile `bcinr-cmca` consumes — is a separate,
  already-executed track (`/Users/sac/mfw/tools/cmca-generator/generator.py`). This spec is
  scoped to tool selection, not to `bcinr-cmca`'s allocator profile.

**Decision:** build on the real, tested implementation (`bcinr-logic` + `bcinr-powl`), not the
untested/fabricated-claim one (`mfw-auto-select`). `mfw-auto-select` is left untouched.

## Goal

Produce one real, independently-reproducible, tested path from an admitted RDF graph to a
selected tool, using the existing tested primitive as-is wherever possible.

## Non-goals (explicitly deferred)

- The 7-coordinate weighted-geometric-mean scoring formula and `q`-exponent lens from the
  design doc. `AutoSelectInput8` already reduces each candidate to one `u32` — a richer
  multi-coordinate mass function can be layered on later without changing this bridge's shape.
- `mfw-auto-select`'s POWL/epoch/chaos/trace/causal-buffer machinery.
- Retracting or re-deriving the fabricated `mfw-auto-select` audit docs (named above, separate
  task).
- Full SHACL/ShEx/QUDT closure over arbitrary ontologies — this bridge handles the bounded
  8-candidate tool-selection case only, consistent with `AutoSelectInput8`'s fixed-size shape.

## Architecture

Three new pieces; everything else is reused as-is.

```
mfw-meaning::shacl::validate_turtle   (real, exists: src/shacl.rs)
        │  per-candidate conformance check
        ▼
[NEW] mfw-meaning::tool_selection     (new module: RDF facts -> masses + eligibility)
        │  produces (tool_id: u8, mass: u32) per eligible candidate
        ▼
bcinr_logic::autonomic::auto_select::AutoSelectInput8   (real, tested, unmodified)
        │  { candidates: [u32; 8], valid_mask: u8 } .select_optimal()
        ▼
bcinr_powl::auto_select_bridge::powl_bridge_select   (real, tested, unmodified)
        │  folds AutoSelectResult into PowlRunState
        ▼
(consumer: PowlRunState.active_mask reflects the selected tool)
```

### New module: `mfw-meaning::tool_selection`

Given:
- one Turtle string per candidate tool (its declared facts: capability, evidence class,
  determinism, latency, etc. — the request's requirements folded in as shared context or
  additional shape constraints)
- one Turtle SHACL shapes string (the eligibility rules — e.g. "reject if
  `requiresDeterminism = true` and candidate `deterministic = false`", mirroring the worked
  example's `VectorToolInputShape`)

Compute, for up to 8 candidates:
1. **Eligibility**: call `mfw_meaning::shacl::validate_turtle(candidate_turtle, shapes_turtle)`
   once per candidate; `report.conforms` becomes that candidate's bit in `valid_mask`. A
   candidate that fails to parse or errors during validation is treated as ineligible (bit
   unset), not a panic — `validate_turtle` already returns `Result`.
2. **Mass**: a single deterministic `u32` per eligible candidate. First-cut rule (documented,
   not hidden): read one already-admitted numeric fact per candidate (e.g. a declared
   `reliabilityPpm`-style literal) directly out of the Turtle data via a small, explicit parse —
   no floating-point geometric mean, no weight table. If a richer multi-factor mass is wanted
   later, it replaces this function's body only; the shape it must produce (`u32`) does not
   change.
3. Emit `AutoSelectInput8 { candidates: [u32; 8], valid_mask: u8 }`, zero-padding unused slots,
   ready to hand directly to `.select_optimal()`.

No new types are added to `bcinr-logic` or `bcinr-powl` — the existing `AutoSelectInput8` shape
is the target, not a new intermediate struct.

## Data flow (worked example)

Using the original 3-tool request (`SPARQLQueryTool`, `RulesEngineTool`, `VectorSearchTool`,
requester requires determinism):

1. Three candidate Turtle fragments + one shapes Turtle (the `VectorToolInputShape`-equivalent:
   `sh:property [ sh:path :requiresDeterminism ; sh:hasValue false ]` marks the vector tool
   ineligible when the request requires determinism).
2. `tool_selection` calls `validate_turtle` three times → `valid_mask = 0b011` (SPARQL=bit0,
   Rules=bit1, Vector=bit2 excluded).
3. Masses read from each candidate's declared reliability fact → e.g. `[250, 252, 0]` (0 masks
   out; irrelevant since bit2 is already unset in `valid_mask`).
4. `AutoSelectInput8 { candidates: [250, 252, 0, 0, 0, 0, 0, 0], valid_mask: 0b011 }
   .select_optimal()` → `AutoSelectResult { is_ok: 1, tool_id: 1, refusal_code: 0 }` (Rules
   tool wins on mass).
5. `powl_bridge_select(&result)` → `u64` mask folded into `PowlRunState` via
   `powl_admit_selection`.

## Error handling / refusal semantics

Reuse `AutoSelectResult`/`AutoSelectRefusal` as-is — no new error type. `tool_selection`'s own
failure modes (Turtle parse error, SHACL engine error) surface as `Result<_, anyhow::Error>` at
the compiler boundary, consistent with `mfw_meaning::shacl`'s existing error style, and are
mapped to "ineligible" (masked out) rather than propagated as a panic into the selection step —
matching this repo's branchless/no-panic discipline on the selection side while keeping the
admission side's existing `Result`-based error handling untouched.

## Testing

One real integration test in `mfw-meaning` (or a new `mfw-auto-select-bridge`-style test crate,
decided during implementation planning): admit the 3-tool fixture above through the real
`shacl::validate_turtle`, compile masses, call the real `bcinr_logic::autonomic::auto_select`
and `bcinr_powl::auto_select_bridge` functions, assert the Rules tool is selected and the Vector
tool is excluded. No mocks or stubs of any of the three crates' real types — same discipline as
this repo's `jtbd_*` test files. A second test asserts an all-ineligible graph (e.g. every
candidate requires unmet authority) yields `AutoSelectRefusal::ControlStateUnadmitted` via the
existing refusal path, not a panic.

## Open questions for implementation planning

- Exact crate/module placement for `tool_selection` (new file in `mfw-meaning`, or a new small
  crate depending on it) — implementation planning should confirm against mfw's existing
  crate-boundary conventions.
- Whether `mfw-meaning` should gain a `bcinr-logic` path dependency directly, or whether the
  compiler's output type should be consumed one layer up (e.g. in `mfw-planner`) to keep
  `mfw-meaning` free of a `bcinr` dependency — a real architectural choice, not decided here.
- The specific Turtle vocabulary/predicate names for the "mass fact" read in step 2 of the
  compiler — the worked example above is illustrative, not a finalized ontology.
