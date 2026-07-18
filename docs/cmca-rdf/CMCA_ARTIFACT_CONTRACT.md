# CMCA Artifact Contract — Gamma_CMCA v1

**Status:** v1.1, DRAFT — pending independent review
**Kind:** mutable current-design document (describes a concrete, versioned artifact format; not a
timeless invariant). Future incompatible changes bump `schema_version` and update *this same
file* — do not fork a new document per version.

## 1. Purpose

Defines the deterministic, digest-bound ARTIFACT boundary between `mfw` (RDF admission +
generation, producer) and `bcinr-cmca` (branchless runtime/numeric kernel/authority machine,
consumer). The boundary is a set of files, not a Cargo dependency: `bcinr-cmca` never links
against `oxigraph`, SHACL/ShEx/N3 tooling, or Python. It only reads the three files this contract
defines and checks their internal consistency.

## 2. The three artifact files

Contract version **Gamma_CMCA v1** consists of exactly three files, always produced and
consumed together as one unit:

| File | Role |
|---|---|
| `cmca_generated.rs` | Rust source: generated constants/tables. Replaces today's `src/generated/*.rs` (`case_studies.rs`, `generalization.rs`, `stability_profile.rs`). |
| `cmca_generation_manifest.json` | Human/tool-readable metadata: schema version, dimension bounds, the six named digests (§3), and provenance fields (generator invocation, timestamp, source graph identifiers). |
| `cmca_generation_receipt.json` | BLAKE3-chained receipt of the generation run. |

### 2.1 Receipt shape (expected, not yet verified against mfw source)

`cmca_generation_receipt.json` SHOULD carry the same digest-chain discipline `mfw-meaning`
already uses for its own admit-graph → validate → receipt pipeline. This document has not read
the `mfw-meaning` receipt implementation directly; the shape below is the expected mirror of that
discipline, to be confirmed against `mfw-meaning` source during migration step 3+:

```json
{
  "chain": [
    { "step": "admit_graph",   "digest": "blake3:...", "prev": null },
    { "step": "validate",      "digest": "blake3:...", "prev": "blake3:..." },
    { "step": "generate",      "digest": "blake3:...", "prev": "blake3:..." },
    { "step": "emit_artifact", "digest": "blake3:...", "prev": "blake3:..." }
  ],
  "final_digest": "blake3:...",
  "schema_version": 1
}
```

Each step's digest covers that step's output bytes; each step's `prev` links to the previous
step's digest, forming a hash chain. `final_digest` covers the full chain. bcinr does not
recompute this chain (it has no access to the RDF graph or generator); it only checks that the
receipt is present, well-formed, and that its `schema_version` matches the manifest's (§4).

## 3. Conceptual `CmcaGeneratedProfile` fields

The manifest (and, transitively, the constants emitted into `cmca_generated.rs`) represent the
following fields conceptually. **These may be realized as compile-time constants in
`cmca_generated.rs` rather than a literal Rust struct at runtime** — the contract binds which
identities must be present and checkable, not the exact Rust representation.

- `schema_version: u32` — Gamma_CMCA contract version this artifact was generated against.

### 3.1 Canonical digest list (binding — exactly six)

The six named digests below are the **entire** binding list. All are `blake3:<hex>` (BLAKE3,
lowercase hex, `blake3:` prefix). No other digest is part of this contract unless added to this
list by a `schema_version` bump (§6).

1. **`rdf_digest`** — BLAKE3 of the admitted RDF graph, serialized as canonical N-Quads: quads
   sorted lexicographically by their full quad string (subject, predicate, object, graph —
   N-Quads text form, not term-by-term), UTF-8 encoded, LF line endings, no trailing whitespace
   on any line and no trailing blank line at end of file.
2. **`admission_digest`** — BLAKE3 of a canonical JSON encoding (§3.2) of the
   admission/validation pass **result**, not the graph itself: which SHACL/ShEx shapes were
   checked, each shape's own content digest, and the pass/fail outcome per shape plus the
   overall outcome.
3. **`generator_digest`** — BLAKE3 of the generator source file(s), concatenated in a fixed,
   declared order (the order MUST be recorded in the manifest alongside this digest), as UTF-8
   bytes exactly as committed to the generator's source tree (no re-encoding, no whitespace
   normalization beyond what is already committed).
4. **`numeric_profile_digest`** — BLAKE3 of a canonical JSON encoding (§3.2) of: the Decimal
   precision, the rounding mode, the min/max representable range, and the Q16.16 scale factor.
5. **`formula_registry_digest`** — BLAKE3 of a canonical JSON encoding (§3.2) of every
   formula/floor identity name in force, paired with its defining expression or
   table-generation rule (e.g. `UniformLeafFloor`, `UniformLeafFloorQ16Residual`).
6. **`generated_payload_digest`** — BLAKE3 of the exact bytes of `cmca_generated.rs` as emitted:
   UTF-8, LF line endings, no post-hoc reformatting between emission and digesting.

**Gap rule:** if some shape identity, registry identity, or other semantic input is not
transitively bound into one of these six digests, a **seventh** named digest MUST be added to
this list (with a `schema_version` bump per §6) — it must never be assumed covered by
proximity to an existing digest. Identifying such a gap, if one exists, is deferred to whichever
agent discovers it during implementation; this document does not claim the six-digest list is
already known to be gap-free, only that it is the current binding list and that gaps are closed
by extension, not by silent reinterpretation of an existing digest's scope.

### 3.2 Canonical serialization rules

These rules apply to every "canonical JSON encoding" referenced above (digests 2, 4, 5) and, for
the RDF case (digest 1), the analogous canonical N-Quads rule already stated in digest 1:

- **Digest algorithm:** BLAKE3 for all six digests. No other hash algorithm is permitted anywhere
  in the digest list.
- **Canonical JSON** means an RFC 8785-style (JCS) encoding:
  - object keys sorted lexicographically (byte-wise on the UTF-8 key), recursively at every
    nesting level;
  - no insignificant whitespace (no space after `:` or `,`, no indentation, no trailing newline);
  - UTF-8 encoding throughout;
  - integers are bare decimal literals with no leading zeros (`0` alone is the only valid
    zero-representation) and no leading `+`;
  - **no floating-point values anywhere in digested JSON.** Any fixed-point quantity (e.g. a
    Q16.16 value) is represented as an integer together with an explicit, separately declared
    scale factor from the `numeric_profile` (§3.1 item 4) — never as a JSON number with a
    decimal point or exponent.
- **Array order is preserved, not sorted.** Arrays inside digested content (e.g. a leaf list, a
  formula-application order) carry semantic meaning in their generation order (leaf order,
  registration order); canonicalization sorts object keys, never array elements.
- **Paths are relative and host-independent.** Any file path appearing inside digested content
  is relative to a declared root, never absolute, and contains no username, no machine
  hostname, and no other host-identifying fragment. A path that varies by toolchain or host
  location MUST NOT appear inside any digested payload.
- **Line endings are LF.** All text digested under this contract (N-Quads, canonical JSON,
  `cmca_generated.rs`) uses `\n` line endings; any `\r\n` present in an upstream source is
  normalized to `\n` before digesting, not after.

Bounded tables and registries the runtime needs, emitted as data in `cmca_generated.rs`:

- `leaf_floor_base` / `leaf_floor_remainder` — the base-`q` + residual-`r` conservation scheme
  (base quotient plus exact remainder, so `base * divisor + remainder` reconstructs the dividend
  exactly) that **replaces** the current rounded-reciprocal `LEAF_RECIP` table
  (`crates/bcinr-cmca/src/allocator.rs`, `const LEAF_RECIP: [NonNegativeFixed; 9]`). This removes
  the rounding-error surface that a reciprocal-multiply approximation carries.
- Bounded object registry, factor registry, measure registry, and lens registry — the fixed-size
  lookup tables the runtime indexes into at O(1)/O(log n), each sized to the `N`/`F`/`K`/`Q`
  dimension bounds declared in the manifest (§4).

## 4. bcinr obligation — `VerifyGeneratedProfile`

At build or test time (a `#[test]` or a `build.rs` check), `bcinr-cmca` MUST verify the artifact
**without invoking any RDF/generation logic** — no `oxigraph`, no SHACL/ShEx/N3, no Python, no
network access. The check reads only the manifest and receipt JSON plus the generated Rust
source's declared array lengths. It verifies:

1. `schema_version` in the manifest is a recognized value (member of the set this document's
   §6 enumerates for the current contract line).
2. The `N`/`F`/`K`/`Q` dimension bounds recorded in the manifest match what the generated arrays
   in `cmca_generated.rs` actually declare (e.g. manifest says `N = 9` leaf floors ⇒
   `leaf_floor_base` and `leaf_floor_remainder` are each length 9).
3. The manifest's digests are internally consistent — in particular, `generated_payload_digest`
   actually matches a freshly computed BLAKE3 hash of `cmca_generated.rs`'s bytes at verification
   time.

**Failure discipline:** an unrecognized `schema_version` or any digest/bound mismatch MUST
produce a typed build/test failure — a `compile_error!`-style refusal at build time, or a failing
verification test at test time. Silent best-effort continuation (falling back to defaults,
warning-and-proceeding, or partial acceptance) is not a permitted outcome under this contract.

## 5. mfw obligation — `Generate`

`mfw` (specifically `mfw-meaning`, invoking the relocated generator) MUST:

1. Admit the CMCA RDF graph via `mfw-meaning`-style admission (admit → validate), recording
   `rdf_digest` and `admission_digest`.
2. Run the generator deterministically. The generator is relocated Python for this release
   (upgraded from its current bcinr-side form); a Rust/ggen port is fenced to a later contract
   version, not this one.
3. Emit all three artifact files (§2) with all six digests (§3) populated — no field left as a
   placeholder or zero digest.
4. Copy/commit the three files into the `bcinr-cmca` tree at:

   ```
   crates/bcinr-cmca/generated-artifact/cmca_generated.rs
   crates/bcinr-cmca/generated-artifact/cmca_generation_manifest.json
   crates/bcinr-cmca/generated-artifact/cmca_generation_receipt.json
   ```

   **Location choice and justification:** a dedicated `generated-artifact/` directory (not
   `src/generated/`) is chosen over reusing the existing `src/generated/mod.rs` re-export pattern
   (`case_studies.rs`, `generalization.rs`, `stability_profile.rs`) because those existing files
   are hand-written Rust that happens to hold generated-looking data, re-exported through normal
   `pub mod` declarations authored by bcinr engineers. The Gamma_CMCA artifact is categorically
   different: it is produced by a separate repository (`mfw`) and must never be hand-edited in
   `bcinr-cmca`. Keeping it out of `src/` and out of the existing `src/generated/mod.rs`
   re-export chain makes "this file is an external artifact, do not edit" structurally visible
   from the path alone, and lets `VerifyGeneratedProfile` (§4) glob a single directory rather than
   distinguish generated-artifact files from hand-written ones inside a shared `src/generated/`.
   Wiring `crates/bcinr-cmca/generated-artifact/cmca_generated.rs` into the crate's module tree
   (e.g. via `include!` or a thin `pub mod` shim in `src/generated/mod.rs` pointing at it) is left
   to the migration step that actually introduces the file, since this document does not create
   or modify any source file.

## 5.1 Identity vs. event separation

`generated_payload_digest` and every digest recorded in `cmca_generation_manifest.json` (all six
of §3.1) are **identity** artifacts: for the same admitted RDF input and the same declared
generator toolchain coordinate, they MUST be byte-identical across machines, across runs, and
across time. Concretely:

- None of the six digests, nor the manifest as a whole, may depend on wall-clock time, host file
  system path, username, process ID, hostname, or any other run-local or machine-local value.
- Running generation twice from the same admitted graph and the same generator source, on two
  different machines, MUST produce a byte-identical `cmca_generation_manifest.json` and a
  byte-identical `cmca_generated.rs`.

`cmca_generation_receipt.json` is the one artifact in Gamma_CMCA that is explicitly permitted —
expected — to vary run-to-run. It is an **event** record, not an identity record: it carries a
timestamp, a toolchain version string, a host-agnostic build coordinate, and a reference (by
digest) to the manifest/payload digests it attests to. Two receipts produced from identical
inputs on two different runs may differ in their timestamp and chain digests while referencing
identical `generated_payload_digest`/manifest digests — **this is expected and correct, not a
nondeterminism defect.** `VerifyGeneratedProfile` (§4) MUST NOT treat receipt-to-receipt
variation as a verification failure; it checks the receipt's internal well-formedness and its
reference to the manifest's digests, not receipt-to-receipt equality across runs.

## 6. Versioning and refusal rules

- `schema_version` is a `u32` field in `cmca_generation_manifest.json`.
- `schema_version` MUST increment on any incompatible change to: the three-file set (§2), the
  six named digests (§3.1), the canonical serialization rules (§3.2), the bounded table shapes
  (§3), or the `VerifyGeneratedProfile` obligations (§4).
- `bcinr-cmca` MUST refuse — not attempt — to consume an artifact whose `schema_version` it does
  not recognize. Refusal is a typed failure (a typed error value or a failing verification test,
  per §4) — best-effort parsing of an unrecognized schema shape is not a permitted outcome.
- Recognized `schema_version` values under this document, as of this draft: `{1}`. Any future
  value MUST be added to this set only by editing this document in place (not by creating a
  `CMCA_ARTIFACT_CONTRACT_v2.md` or similar fork).
- This document itself is versioned in its title and status line above; a `schema_version` bump
  requires updating this same file's §3 (fields), §4 (verification obligations), and this §6
  (recognized-version set) together, in the same change.

## 8. Correspondence domain classification

Correspondence verification (comparing new-generator output against the pre-migration baseline,
see `crates/bcinr-cmca/tests/fixtures/PRE_MIGRATION_BASELINE.md`) MUST classify every fixture
into **exactly one** of the following three categories before any comparison is made. No fixture
may be left unclassified, and no fixture may be split across categories.

1. **`CORRESPONDENCE_REQUIRED`** — the old generator's output for this fixture was lawful. The
   new generator MUST reproduce it byte-for-byte. A mismatch here is a regression.
2. **`DEFECTIVE_BEHAVIOR_QUARANTINED`** — the old generator's output for this fixture reflected a
   known defect (examples: a zero-default fallback on missing input, a cycle silently returning
   zero instead of refusing, binary-float rounding error). This output MUST NOT be reproduced.
   The new generator MUST instead either produce a typed refusal or a specified, explicitly named
   corrected value — the correction target must be named in the fixture's classification, not
   left implicit.
3. **`NEW_LAW_REQUIRED`** — the fixture exercises a construct with no precedent in the old
   generator, requiring a freshly authored admission law (examples: index-injectivity
   enforcement, cycle refusal itself as a new obligation rather than a corrected value).

Whoever runs correspondence verification MUST perform this three-way classification for every
fixture before comparing, and MUST NOT treat `DEFECTIVE_BEHAVIOR_QUARANTINED` bytes as a
pass/fail equality target — a "match" against quarantined defective output is not a pass, and a
"mismatch" against it is the expected, correct result.

## 9. Explicit non-scope of this document

This document is specification-only for migration step 2. It does **not**:

- modify any existing `bcinr-cmca` source file (including `allocator.rs`, `src/generated/mod.rs`,
  or any of `case_studies.rs` / `generalization.rs` / `stability_profile.rs`);
- create `cmca_generated.rs`, `cmca_generation_manifest.json`, or `cmca_generation_receipt.json`;
- confirm the mfw-meaning receipt shape (§2.1) against mfw source — that confirmation is a later
  migration step.

## See Also

- `docs/cmca-rdf/V26_7_17_RELEASE_LEDGER.md` — current status of this migration (REPORTED items
  only, tagged per the control-plane separation law).
- `.claude/rules/cmca/rdf-generation.md` — timeless invariants for RDF generation.
- `.claude/rules/cmca/packaging.md` — timeless invariants for packaging.
- `.claude/agents/cmca-semantics.md`, `.claude/agents/cmca-verifier.md`,
  `.claude/agents/cmca-release-integrator.md` — stable authority definitions for the agents
  operating under this contract.
