# bcinr-release

`bcinr-release` is the fail-closed release-admission tool for BCINR. It does not implement temporal planning, POWL execution, scheduling, resource law, OCEL semantics, swarm behavior, or another runtime capability. Those remain owned by their authoritative crates.

## Jurisdiction

This tool owns the transition from an exact repository tree to a replayable release-verification receipt:

```text
exact tree
  -> validated release profile
  -> bounded verifier rails
  -> retained stdout/stderr evidence
  -> artifact admission
  -> byte-identity proof
  -> exact standing
  -> BLAKE3 receipt
```

The tool may execute capability tests, but it may not redefine capability semantics or convert a capability failure into a release success.

## Design laws

- No shell is used to execute profile commands.
- Programs and arguments are represented separately.
- Every rail has a bounded timeout.
- Standard output and standard error are retained separately.
- Every retained log is BLAKE3-digested.
- The exact Git head, branch, remote, toolchain, operating system, architecture, and tree status are captured.
- A dirty input tree, changed head, changed tracked tree, repository mismatch, or exact-head mismatch blocks admission.
- Required artifacts must be regular files and receive BLAKE3 digests.
- Manufactured and consumed files are compared byte-for-byte, not by names or reported versions.
- Evidence is emitted only below `target/`.
- `ALIVE` is calculated from observed results. It is never accepted from a report field or command output.

## Standing

- `ALIVE`: every required rail passed, every required artifact exists, every required identity pair is byte-identical, and repository admission remained valid.
- `PARTIAL_ALIVE`: every required condition passed but an explicitly optional condition failed.
- `BUILD_BROKEN`: at least one required executable rail failed or timed out.
- `BLOCKED`: repository identity, exact-head, clean-tree, artifact, or byte-identity admission failed.

The process exits with code `0`, `3`, `1`, or `2`, respectively.

## Invocation

```bash
cargo run --locked -p bcinr-release -- \
  verify \
  --profile release/v26.7.28/production.json \
  --expected-head "$(git rev-parse HEAD)" \
  --output target/release-evidence/v26.7.28
```

The output directory must be repository-relative and located under `target/`.

## Evidence

A run emits:

```text
target/release-evidence/v26.7.28/
  logs/
    <rail>.stdout.log
    <rail>.stderr.log
  receipt.json
  receipt.blake3
```

`receipt.json` records each command vector, working directory, timeout outcome, exit code, duration, log digest, artifact digest, identity comparison, provenance observation, admission issue, and final standing.

## Version policy

The executable is generic. Version-specific requirements belong under `release/<version>/`. Adding a release must not hardcode expected counts, benchmark values, checksums, or success states into Rust or shell source.
