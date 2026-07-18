# Package Reality Report — Track E (Package and Artifact Reality)

**Version/milestone:** v26.7.17 | **Generated (UTC):** 2026-07-18T04:03:06Z |
**Git branch:** `recovery/cmca-v26.7.17-c2` | **Git HEAD:** `7e7e7cd5`

This report records what was actually run, on this machine, against the current
working tree, for Track E. It is reconciled against real command output below;
no finding in this report is asserted without a command transcript. Companion
mutable evidence: `crates/bcinr-cmca/PACKAGE_REALITY_RECEIPT.md` (regenerated
by `cargo make package-reality-check` each run — read that file for the raw
run-output block of the task run reported in §1).

## 1. `cargo make package-reality-check` — genuinely rerun

- `scripts/gates/package-reality-check.sh` exists on disk, is on `bash -n`
  syntactically valid, and is **not git-tracked** (`git status --porcelain`
  reports `?? scripts/gates/package-reality-check.sh`). Per the task
  instructions this is acceptable at this stage — a later commit phase
  handles committing it. It is present and coherent, not shell history: it is
  a real, structured 245-line script with a documented exit-code contract,
  and it matches the `[tasks.package-reality-check]` entry in `Makefile.toml`
  (also currently uncommitted — `Makefile.toml` shows as `modified`, i.e.
  tracked with a pending diff) in both mechanics and prose.
- I ran `cargo make package-reality-check` for real (not read-only inspection
  of the file). It completed in 4.17s, exit code 0, and **regenerated**
  `crates/bcinr-cmca/PACKAGE_REALITY_RECEIPT.md` with a fresh timestamp
  (`2026-07-18T04:00:44Z`) and this run's own raw output, confirming the
  Makefile→script→receipt pipeline is live end-to-end, not aspirational.
- Verdict: **ALIVE** as a replayable gate. (Distinct claim from whether
  packaging itself succeeds — see §3.)

## 2. `cargo package -p bcinr-logic --locked`

Ran directly (not just via the wrapper script) to double-check:

```
cargo package -p bcinr-logic --locked
   Packaging bcinr-logic v26.7.17 (/Users/sac/bcinr/crates/bcinr-logic)
    Updating crates.io index
    Packaged 701 files, 2.4MiB (348.6KiB compressed)
```

- **PASSED cleanly — no `--allow-dirty` retry needed.** (cargo's dirty-tree
  check is scoped to files inside the crate's own directory; `bcinr-logic/`
  itself had no uncommitted changes even though the wider workspace tree
  does.)
- Artifact: `target/package/bcinr-logic-26.7.17.crate`, 356,984 bytes (348.6
  KiB, matching cargo's own report), 701 files per `tar tzf`.
- sha256, independently recomputed with `shasum -a 256` (not just trusted
  from the script's own log):
  `0adc567782eb590edb921acb7549ddc6876b20fc4f76f22af322bd291513a6cd` —
  matches the digest the script recorded in
  `target/package-reality/digests.txt` and in the receipt.

## 3. `cargo package -p bcinr-cmca --locked` — known blocker, distinguished from unexpected failure

```
cargo package -p bcinr-cmca --locked
error: 24 files in the working directory contain changes that were not yet committed
  -> retried with --allow-dirty (per script logic; this makes the result an
     INTERIM SMOKE CHECK, not release-closing evidence)
error: failed to prepare local package for uploading
Caused by:
  failed to select a version for the requirement `bcinr-logic = "^26.7.17"`
  candidate versions found which didn't match: 26.6.24, 26.4.22, 26.4.21, ...
  location searched: crates.io index
  required by package `bcinr-cmca v26.7.17 (/Users/sac/bcinr/crates/bcinr-cmca)`
```

- This is **exactly** the known, already-diagnosed sequencing blocker:
  `cargo package` rewrites bcinr-cmca's path dependency on bcinr-logic into a
  registry requirement (`^26.7.17`), and bcinr-logic 26.7.17 is not published
  to crates.io. No unexpected failure signature was observed.
- Note on evidence class: because bcinr-cmca's own directory has 24
  uncommitted files, this result required the script's `--allow-dirty`
  retry, so it is labeled an **interim smoke check**, not release-closing
  evidence, per `.claude/rules/cmca/packaging.md` — separately from (and in
  addition to) the sequencing blocker itself. Re-running against a fully
  committed `crates/bcinr-cmca/` tree would remove the dirty-tree caveat but
  would not change the sequencing-blocker outcome, since that failure is a
  registry-resolution error, unrelated to git cleanliness.
- No `.crate` tarball was produced by this command this run.

## 4. Extraction/build-from-tarball — SKIPPED, honestly

- Instruction 4's condition ("if a bcinr-cmca tarball IS produced") did not
  hold this run: §3 produced no tarball, so the script's step (c) correctly
  reported `SKIPPED (no tarball produced)` and no extraction/build/test was
  performed.
- `target/package/` does contain a stale `bcinr-cmca-26.6.24.crate`
  (mtime `Jul 17 19:12`, predating this run's `21:00` bcinr-logic build), a
  leftover from a prior local run at an **older version** than the current
  `crates/bcinr-cmca/Cargo.toml` (`26.7.17`). I did not extract or build from
  it: doing so would test a stale, different-version artifact, not the
  current source tree, and would not be honest evidence for this release.
  It was not deleted (fix-forward-only / no destructive cleanup).
  sha256 (independently recomputed):
  `34ee8a92bb8966ca8ee8513fe24e4e0c78dab7aaef447f26ad87d7d1e5de51d1`.
- This sub-step is **blocked-pending** on §3's known sequencing blocker (not
  itself a defect, and not evaluated as a separate technical failure — see
  Overall Disposition below).

## 5. Generated CMCA artifacts/manifests in the package listing

Since `cargo package -p bcinr-cmca --locked` fails at the
registry-dependency-resolution stage (§3) before it reaches file listing or
verification, I additionally ran the **file-manifest-only** command, which
does not invoke the path→registry dependency rewrite and so is not blocked
by §3:

```
cargo package -p bcinr-cmca --list --locked --allow-dirty
```

(`--allow-dirty` still required for the same 24-file dirty-tree reason as
§3; this is a distinct, lighter-weight evidence class — a manifest listing
derived from `Cargo.toml` include/exclude rules and VCS file state, **not** a
build-verified package — and should not be conflated with a successful
`cargo package`.)

Result: 159 files listed. `generated-artifact/**` is present and included:

```
generated-artifact/PRODUCER_REPRODUCTION.md
generated-artifact/case-studies/cmca_generated.rs
generated-artifact/case-studies/cmca_generation_manifest.json
generated-artifact/case-studies/cmca_generation_receipt.json
generated-artifact/generalization/cmca_generated.rs
generated-artifact/generalization/cmca_generation_manifest.json
generated-artifact/generalization/cmca_generation_receipt.json
```

Verdict: generated-artifact inclusion is **confirmed at the manifest-listing
level**. It is **not** confirmed at the build-verified-package level, because
no such package could be produced this run (§3).

## 6. `cargo tree -p bcinr-cmca` — dependency-graph audit

Full tree (`cargo tree -p bcinr-cmca --locked`, dev+build included):
grepped case-insensitively for `mfw`, `oxigraph`, `praxis-graphlaw`, `rdf` →
**no matches**.

Production-only tree (`cargo tree -p bcinr-cmca --locked -e normal,build`):

```
bcinr-cmca v26.7.17 (/Users/sac/bcinr/crates/bcinr-cmca)
└── bcinr-logic v26.7.17 (/Users/sac/bcinr/crates/bcinr-logic)
```

Cross-checked directly against `crates/bcinr-cmca/Cargo.toml`:

- `[dependencies]` — exactly one entry: `bcinr-logic = { path = "../bcinr-logic", version = "26.7.17" }`. No mfw, RDF, or graph-processing dependency anywhere in production dependencies.
- `chicago-tdd-tools = { version = "26.7.1", ... }` is declared **only** under `[dev-dependencies]`; it does not appear under `[dependencies]`. Confirmed both from the manifest text and from the `[dev-dependencies]` branch of the full `cargo tree` output.
- `chicago-claims` — **precision correction**: this package is not a dependency of bcinr-cmca at all, in either section. It does not appear in `crates/bcinr-cmca/Cargo.toml` (checked both dependency tables) and does not appear anywhere in the whole-workspace `Cargo.lock` (`grep -n "chicago-claims" Cargo.lock` → no matches). The instruction's premise that it "appears... only under dev-dependencies" is not what was found; the accurate statement is that it is absent from this crate's dependency graph entirely, which is a strictly stronger (not weaker) result for the "never under normal `[dependencies]`" property being checked.

Verdict: **confirmed** — mfw/RDF/graph-processing absent from bcinr-cmca's dependency graph (prod and dev); chicago-tdd-tools dev-only; chicago-claims absent entirely.

## 7. mfw filesystem-absence — honest, not closed

- `/Users/sac/mfw` was confirmed present on this filesystem (`ls -d
  /Users/sac/mfw` succeeds) — it is a real, non-generated checkout, not a
  fixture.
- What is proven: **dependency-graph absence** — `cargo tree -p bcinr-cmca`
  contains zero references to mfw/oxigraph/praxis-graphlaw/RDF, at any
  dependency edge (normal, build, or dev), as shown in §6. This is real,
  reproducible evidence.
- What is **not** proven, and this task will not attempt to prove: whether
  bcinr-cmca would still build correctly on a machine where
  `/Users/sac/mfw` does not exist on disk at all (true filesystem absence).
  Testing that would require deleting or renaming the real
  `/Users/sac/mfw` checkout, which is destructive, out of this track's
  scope, and was not done. This gap is stated plainly and is **not**
  presented as closed.

## 8. `cargo publish` — not run

No `cargo publish` command, dry-run or otherwise, was executed at any point
in this track. Only `cargo package` (with `--locked`, `--list`,
`--allow-dirty` variants), `cargo tree`, `shasum`, `tar tzf`, and `cargo make
package-reality-check` were run. No git commit, branch change, or
destructive operation was performed; no files outside this track's writable
region (`scripts/gates/**`, `Makefile.toml`, and this pair of report files)
were modified. `git status --porcelain` before and after this session shows
the identical set of tracked-file modifications (`Cargo.lock`,
`Makefile.toml`, `crates/bcinr-cmca/{Cargo.toml,src/allocator.rs,src/mode_switch.rs,tests/hostile_mutants.rs}`,
`docs/cmca-rdf/V26_7_17_RELEASE_LEDGER.md`) — none introduced by commands run
in this track.

## Summary table

| Check | Result | Evidence class |
|---|---|---|
| `package-reality-check` script+task genuinely present and runs | ALIVE | live execution, exit 0, regenerated receipt |
| `cargo package -p bcinr-logic --locked` | PASS (clean, no dirty retry) | build-verified package, sha256-digested |
| `cargo package -p bcinr-cmca --locked` | FAIL — known sequencing blocker only, no unexpected failure | build attempt (interim smoke check, dirty-tree retry) |
| Extract+build from bcinr-cmca tarball | SKIPPED (no tarball produced; correctly not fabricated) | n/a — conditional on above per instruction 4 |
| `generated-artifact/**` in package listing | CONFIRMED | manifest-listing only (`--list`), not build-verified |
| mfw/RDF/graph-processing absent from prod deps | CONFIRMED | `cargo tree` dependency-graph evidence |
| chicago-tdd-tools dev-only | CONFIRMED | manifest + tree cross-check |
| chicago-claims dev-only | **Not applicable — absent entirely**, not present in either section | manifest + workspace Cargo.lock grep |
| mfw true filesystem-absence | **NOT tested, not claimed** | stated limitation only |
| `cargo publish` (any form) | Not run | n/a |

## Overall disposition

No unexpected technical failure was found anywhere in this pass: the gate
script exists and runs reproducibly; bcinr-logic packages cleanly and its
digest is independently verified; bcinr-cmca's only failure matches the
known, already-diagnosed registry-sequencing signature exactly (checked
against the precise error text, not just "it failed"); the generated-artifact
manifest, dependency-graph exclusions, and dev-only classification all
checked out; and every honest limitation (mfw true filesystem-absence,
tarball-extraction being conditional on a tarball existing) is stated as a
limitation, not silently closed or fabricated. Per this task's own framing,
the known sequencing blocker is an orthogonal external-authority gap and
does not by itself make the track BLOCKED.

---

BCINR_PACKAGE_REALITY_ALIVE
BCINR_PUBLISH_SEQUENCE_AUTHORITY_REQUIRED
