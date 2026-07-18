---
paths:
  - "crates/bcinr-cmca/Cargo.toml"
  - "Cargo.toml"
  - "CHANGELOG.md"
---

# Release Packaging

## Law

When a package graph contains an internal path dependency, dry-run publication must be
attempted in true dependency order: a dependency must be published, or independently
dry-run verified, before any dry-run of its dependent is treated as evidence. A dependent's
dry-run success may never be assumed to imply, or substitute for, its dependency's
publishability — each member of the graph must clear its own dry-run.

Final release evidence must come from a clean, fully committed working tree. A `cargo
package` or `cargo publish --dry-run` invocation using an allow-dirty flag (or any
equivalent that tolerates uncommitted or untracked changes) is admissible only as an
interim smoke check during development — never as the evidence cited to close a release
gate.

A package must be verified to build using only its own packaged contents — the file set a
package-list command reports as included — not the surrounding repository working tree. A
package that omits a generated file its build depends on is a packaging defect even when
the ordinary workspace build succeeds, because the workspace build has access to files the
package itself does not carry.

Publishable-crate metadata (license, description, and this repository's convention for
readme, repository, and keywords fields) must be complete before a dry-run is treated as
meaningful evidence of publishability — not merely before an actual `cargo publish`. The
repository's convention for these fields must be confirmed by inspecting an already-complete
sibling crate's `Cargo.toml`, never assumed from general crates.io practice.

## Falsifier

Any of the following observed in the repository or its release process falsifies this rule:

- A dependent crate's dry-run is cited as passing evidence while its internal path
  dependency has not itself been published or independently dry-run verified.
- A release-closing claim whose supporting `cargo package` or `cargo publish --dry-run` run
  used `--allow-dirty` (or ran against a tree with uncommitted or untracked changes).
- A build verification step that compiles against the workspace source tree rather than the
  extracted contents of the package archive (or the output of a package-list command), when
  claiming the package itself is buildable.
- A dry-run treated as informative while the crate's `Cargo.toml` is missing license,
  description, or a field this repository's convention (as evidenced by a complete sibling
  crate) requires.

## Required Evidence

- Dependency-order evidence: a recorded per-crate publish or dry-run result for every
  internal path dependency, ordered so each dependency's result precedes its dependent's.
- Clean-tree evidence: a `git status` (or equivalent) showing no uncommitted or untracked
  changes, taken contemporaneously with the final dry-run or publish cited as closing
  evidence.
- Packaged-contents evidence: the output of a package-list command (e.g. the file list a
  packaging tool reports as included) plus a build performed against exactly that file set,
  not the working tree.
- Metadata-completeness evidence: the crate's `Cargo.toml` fields compared side-by-side
  against a named already-complete sibling crate's `Cargo.toml`, with the convention it
  establishes stated explicitly rather than assumed.

## Standing Consequence

A dry-run or publish result obtained in violation of this rule (out-of-order, dirty-tree, or
workspace-sourced) does not establish publishability and voids any release-closing claim
resting on it. Such a result may be recorded only as an interim smoke check; the release
gate remains open until evidence satisfying this rule's requirements is produced.

## Two-Repository Release Graph

When CMCA semantic generation is owned by a separate release-time producer (e.g. an
external repository or toolchain distinct from the consuming crate's own repository), the
release graph must be:

1. The producer generates the semantic artifact.
2. The generated artifact is committed into the consuming repository (never fetched or
   regenerated implicitly at package/build time).
3. Internal path dependencies within the consuming repository build in true dependency
   order (per the Law above).
4. Dry-run publish is attempted only after 1–3 are satisfied.

The producer repository or toolchain is a release-time producer only. It must never appear
as a Cargo runtime dependency of the published crate — the published crate depends on the
committed artifact, not on the producer. The producer's generation commands (for example a
validate -> generate -> verify-generated sequence, or an equivalent pipeline) must run and
succeed before the consuming crate is packaged. Packaging the consuming crate against a
stale or unregenerated artifact is a packaging defect, independent of whether the ordinary
workspace build happens to succeed.

### Falsifier

Either of the following falsifies this section: the published crate's package contents
include the producer toolchain, or the published crate's `Cargo.toml` declares a runtime
dependency on the producer toolchain or repository.

### Required Evidence

- Package-manifest evidence: the published crate's `Cargo.toml` showing no dependency entry
  (path, git, or registry) on the producer toolchain or repository.
- Dependency-graph evidence: a full dependency-graph inspection (e.g. a `cargo tree` or
  equivalent listing) of the published crate showing the producer is absent from the graph.

## Nonclaims

This rule does not claim, and takes no position on, whether bcinr-cmca and its internal
path dependency must carry a synchronized version number, or whether any existing crates.io
patch override pointing at an absolute local path constitutes a blocking hazard for this
release. Both are open release hypotheses for the release integrator to test against the
live repository state and record in the release ledger — they are not settled policy and
must not be encoded here as such.
