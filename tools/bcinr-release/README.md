# bcinr-release

`bcinr-release` is BCINR's fail-closed release-admission engine. It does not implement temporal planning, POWL execution, scheduling, resource law, OCEL semantics, swarm behavior, or another runtime capability. Those remain owned by their authoritative crates.

## Jurisdiction

The tool owns one transition:

```text
exact repository object
  -> authenticated bounded profile
  -> preflight admission or zero-actuation refusal
  -> hermetic verifier rails
  -> retained and hashed evidence
  -> artifact-tree admission
  -> byte-identity proof
  -> computed standing
  -> atomic release receipt
```

The tool may execute capability verifiers. It may not redefine capability semantics, repair a failed capability implicitly, or convert a capability failure into release success.

## Admission before actuation

No verifier rail starts until all preflight laws are admitted:

- the caller supplies a full 40- or 64-digit expected Git object ID;
- `HEAD` resolves to that exact commit;
- `origin` normalizes exactly to the profile's `owner/name` repository;
- Git object format and commit timestamp are known;
- Rust and Cargo toolchains are observable;
- the tracked and unignored tree is clean;
- recursive submodule state is exact;
- profile and evidence paths remain inside the repository without symlink traversal.

A preflight failure emits skipped rail receipts. It does not execute commands.

## Process-execution laws

Each rail executes without a shell and with:

- separate program and argument vectors;
- cleared ambient environment;
- an explicit inherited-environment allowlist;
- refused dynamic-loader and Rust compiler-wrapper injection variables;
- closed standard input;
- fixed `LC_ALL=C`, `LANG=C`, and `TZ=UTC`;
- deterministic `SOURCE_DATE_EPOCH` from the admitted commit;
- an exact canonical executable target and BLAKE3 digest before and after execution;
- a repository-state digest before and after execution;
- bounded timeout and bounded retained log volume;
- complete stdout/stderr stream hashing even when retained logs are truncated;
- whole-process-group termination on Unix;
- Linux pidfd-backed `wait`, `try_wait`, and termination to avoid PID-reuse races.

A changed executable or repository state stops subsequent rails.

## Evidence laws

Evidence is emitted only under a new, empty `target/` directory. The engine:

- refuses symlink components;
- creates private directories and files;
- acquires a create-new run lock;
- retains stdout and stderr independently;
- supports bounded recursive artifact trees;
- rejects artifact-tree symlinks;
- compares manufactured and consumed files byte-for-byte;
- hashes the verifier executable itself;
- derives domain-separated BLAKE3 hashes for profiles, executables, logs, artifacts, artifact trees, receipts, and the complete evidence graph;
- writes the final receipt with flush, `fsync`, atomic rename, and parent-directory synchronization.

`ALIVE` is derived only from the observed receipt graph. Command output cannot assert its own standing.

## Nightly boundary

The crate intentionally uses three narrowly fenced standard-library nightly features:

- `linux_pidfd` for race-free Linux child supervision;
- `unix_kill_process_group` for descendant-complete timeout termination;
- `unix_send_signal` for the process-group signal operation used by that termination path.

The release profile compiles the crate through `-Z allow-features=linux_pidfd,unix_kill_process_group,unix_send_signal`, preventing unrelated unstable features from entering the verifier.

Cargo nightly rails additionally produce SBOM precursor files and two independently built, path-trimmed release binaries whose bytes must match.

## Profile bounds

Profile schema v2 bounds every externally supplied dimension, including:

- profile bytes;
- rail, artifact, and identity counts;
- argument and environment counts;
- string lengths;
- timeout duration;
- retained log bytes;
- recursive artifact entries;
- minimum artifact-file count.

Unknown JSON fields and duplicate IDs are refused.

## Standing and exit codes

| Standing | Meaning | Exit |
|---|---|---:|
| `ALIVE` | Every required rail, artifact, identity, and repository law passed. | 0 |
| `BUILD_BROKEN` | At least one required executable rail failed or timed out. | 10 |
| `BLOCKED` | Repository, provenance, artifact, byte-identity, or actuation authority was refused. | 20 |
| `PARTIAL_ALIVE` | Required conditions passed but an explicitly optional condition failed. | 30 |
| `UNKNOWN` | The engine could not establish a bounded standing. | 40 |
| `UNSUPPORTED` | The requested verification boundary is not implemented. | 50 |
| verifier refusal | CLI, profile, or evidence-engine failure prevented a receipt. | 70 |

## Invocation

Build the exact verifier first, then invoke that binary directly:

```bash
cargo build --locked -p bcinr-release --bin bcinr-release

target/debug/bcinr-release verify \
  --profile release/v26.7.28/production.json \
  --expected-head "$(git rev-parse HEAD)" \
  --output "target/release-evidence/v26.7.28/$(git rev-parse HEAD)/manual-1"
```

The output directory must be repository-relative, below `target/`, empty, and free of symlink components.

## Evidence layout

```text
target/release-evidence/v26.7.28/<head>/<run>/
  logs/
    <rail>.stdout.log
    <rail>.stderr.log
  evidence.root
  receipt.blake3
  receipt.json
  standing.txt
```

`receipt.json` records command vectors, admitted environment names, canonical executable identity, process outcome, exit signal, timeout, duration, complete-stream log hashes, retained-byte bounds, repository-state hashes, artifact trees, identity comparisons, exact provenance, admission issues, verifier identity, and final standing.

## Observed crate-local standing

The bounded closure run for this PR observed all of the following on `nightly-2026-07-25`:

- canonical Rust formatting;
- workspace lockfile materialization;
- `cargo check -p bcinr-release --all-targets`;
- locked check replay;
- 15 passing unit tests;
- `cargo clippy --locked -p bcinr-release --all-targets -- -D warnings`;
- mutation-bounded publication of only the admitted source and lockfile;
- self-deletion of the temporary write-capable closure workflow.

Therefore the release-admission crate is `ALIVE` at the crate-local compile, unit, and lint boundary. This does not grant `ALIVE` standing to the complete v26.7.28 release, which still requires the permanent production profile, capability integration, Miri, supply-chain, reproducible-build, generated-byte identity, E2E, cross-target, and benchmark receipts.

## Version policy

The executable is generic. Version-specific requirements belong under `release/<version>/`. Adding a release must not hardcode expected counts, benchmark values, checksums, or success states into Rust, workflow, or shell source.
