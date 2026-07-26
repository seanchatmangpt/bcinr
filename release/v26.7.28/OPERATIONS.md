# BCINR v26.7.28 Production Operations

## Governing invariant

```text
zero unreceipted actuation
```

This runbook governs promotion of one exact BCINR v26.7.28 commit. It does not grant production standing to a branch name, local checkout, generated report, successful command copied from another commit, or prior workflow run.

The promoted object is the tuple:

```text
BCINR commit
+ Cargo.lock
+ release profile
+ verifier executable
+ exact ggen manufacturer object graph
+ manufactured contract bytes
+ production-admission receipt
+ target deployment configuration
```

Changing any member creates a new candidate requiring a new receipt.

## Ownership boundary

`tools/bcinr-release` owns release admission only. It does not define temporal planning, POWL semantics, resource law, scheduler law, OCEL semantics, replay semantics, swarm behavior, or capability-local falsification.

The release tool may execute those capabilities' verifiers and aggregate their evidence. It may not repair capability failures implicitly, reinterpret a typed refusal as success, or produce `ALIVE` from missing evidence.

## Release-admission states

| State | Meaning | Promotion consequence |
|---|---|---|
| `ALIVE` | Every required repository, rail, artifact, identity, and evidence law passed. | Candidate may proceed to controlled canary. |
| `BUILD_BROKEN` | A required executable rail failed or timed out. | Promotion refused. |
| `BLOCKED` | Repository identity, provenance, artifact, byte identity, or actuation authority was refused. | Promotion refused. |
| `PARTIAL_ALIVE` | Required laws passed but an explicitly optional condition failed. | Production promotion refused unless a separately authorised canary policy admits that exact exclusion. |
| `UNKNOWN` | Observation was insufficient to establish standing. | Promotion refused. |
| `UNSUPPORTED` | The requested verification boundary is not implemented. | Promotion refused for that boundary. |

A skipped, cancelled, stale, unavailable, or action-required workflow is not a passing workflow.

## Admission engine boundary

### Preflight

No verifier command executes until the release engine admits:

- a full 40- or 64-digit expected Git object ID;
- exact equality between that object and `HEAD`;
- exact normalisation of `origin` to the profile's `owner/name` repository;
- known Git object format and commit timestamp;
- observable Rust and Cargo toolchains;
- a clean tracked and unignored tree;
- exact recursive submodule state;
- a bounded schema-v2 profile;
- a new, empty output directory below `target/`;
- no symlink component in profile, output, working-directory, artifact, or identity paths.

A failed preflight emits skipped-rail records and performs zero command actuation.

### Command execution

Every rail executes:

- without a shell;
- from a separate program-and-arguments vector;
- with standard input closed;
- with the environment cleared before an explicit allowlist is restored;
- with dynamic-loader and Rust compiler-wrapper injection variables refused;
- under `LC_ALL=C`, `LANG=C`, and `TZ=UTC`;
- with `SOURCE_DATE_EPOCH` derived from the admitted commit;
- with a bounded timeout;
- with bounded retained logs and complete-stream BLAKE3 hashes;
- with canonical executable identity hashed before and after execution;
- with repository state hashed before and after execution;
- inside a new Unix process group;
- with Linux pidfd-backed child supervision;
- with whole-process-group termination on timeout.

Executable drift or repository mutation stops subsequent rails.

### Evidence construction

The engine:

- creates private evidence directories and files;
- acquires a create-new run lock;
- records stdout and stderr independently;
- records observed bytes, retained bytes, and truncation state;
- rejects artifact-tree symlinks;
- bounds recursive artifact traversal;
- compares manufactured and consumed files byte-for-byte;
- hashes the release-profile bytes and verifier executable;
- derives domain-separated BLAKE3 hashes for every evidence class;
- derives one evidence root over provenance, rails, artifacts, identities, issues, and standing;
- writes receipts using flush, `fsync`, atomic rename, and parent-directory synchronisation.

Command output cannot assert its own standing. Standing is calculated from the retained receipt graph.

## Nightly and supply-chain boundary

The release verifier intentionally admits only these standard-library nightly features:

- `linux_pidfd`;
- `unix_kill_process_group`;
- `unix_send_signal`.

The policy enforces that set through Cargo's `-Z allow-features` fence.

The production rail also requires:

- exact pinned Rust toolchains;
- SHA-pinned GitHub Actions;
- credential-free permanent checkouts;
- exact ggen and sibling-repository SHAs;
- locked Cargo replay;
- Miri for the release tool;
- Clippy with warnings denied;
- `cargo audit`;
- `cargo deny` advisory, licence, and source policy;
- Cargo nightly SBOM precursor generation;
- two path-trimmed release builds whose bytes must match.

The permanent production workflow has read-only repository permissions. Any temporary write-capable manufacturer must be branch-scoped, mutation-bounded, independently verified, and self-deleting.

## Promotion preconditions

Promotion is refused unless:

- the exact-head production-admission workflow completed successfully;
- `receipt.json` reports `ALIVE`;
- `receipt.blake3` authenticates the retained receipt;
- `evidence.root` matches the receipt's evidence root;
- every required rail passed;
- every required artifact was admitted;
- every required byte-identity relation passed;
- the ggen projections are byte-identical to BCINR's consumed outputs;
- the verifier's own executable hash is retained;
- two independent release builds of the verifier are byte-identical;
- the candidate is mergeable with current `main`;
- no temporary workflow remains in the candidate;
- no unresolved P0 or P1 defect exists;
- the rollback owner has acknowledged the exact candidate and rollback target;
- the target supports every capability claimed for that target.

## Evidence retrieval

The GitHub workflow writes evidence below:

```text
target/release-evidence/v26.7.28/<head>/<run-id>-<attempt>/
```

Expected files include:

```text
logs/
evidence.root
receipt.blake3
receipt.json
standing.txt
```

The workflow artifact is a transport copy, not the authority. Before enterprise retention:

1. verify the artifact's GitHub digest;
2. verify `receipt.blake3` against `receipt.json`;
3. verify `evidence.root` against the receipt;
4. verify the expected head, repository, profile hash, verifier hash, and toolchain provenance;
5. copy the complete artifact to immutable enterprise evidence storage;
6. record storage identity and retention policy in the deployment receipt.

## Promotion sequence

### 1. Exact candidate admission

Record:

- BCINR candidate SHA;
- current `main` SHA and merge base;
- ggen manufacturer SHA;
- every pinned sibling SHA;
- `Cargo.lock` digest;
- release-profile digest;
- verifier executable digest;
- production receipt digest;
- evidence root;
- target environment identifier;
- deployment owner;
- rollback owner and rollback SHA.

### 2. Tenant-isolated dry canary

Deploy the exact admitted bytes with irreversible external actuation disabled or routed to a controlled sink.

Exercise:

- deterministic execution;
- deterministic replay;
- typed refusal;
- evidence export;
- restart recovery;
- duplicate-delivery handling;
- bounded resource exhaustion;
- operational telemetry and alert routing.

Promotion stops on any receipt mismatch, duplicate actuation, panic, deadlock, evidence loss, unbounded resource growth, cross-tenant evidence visibility, or unexplained performance regression.

### 3. Regional production canary

Enable bounded traffic for one region and one tenant class. Preserve a direct rollback path to the last admitted release.

Compare:

- admitted requests;
- refused requests by typed refusal code;
- execution and replay roots;
- resource saturation;
- process restarts and lease recovery;
- latency and throughput against the admitted baseline;
- error-budget consumption;
- evidence completeness and retention latency.

### 4. Multi-region expansion

Expand only after the regional canary produces complete receipts and no release blocker. Validate:

- active-active logical ordering;
- regional failover;
- idempotent retry after ambiguous acknowledgement;
- tenant evidence isolation;
- runtime-version compatibility;
- rollback from each active region.

### 5. Global promotion

Promote the same bytes and configuration that passed canary. Rebuilding, regenerating, or modifying configuration creates a new candidate.

## Mandatory rollback triggers

Rollback is mandatory for:

- unreceipted actuation;
- duplicate irreversible actuation;
- receipt, evidence-root, or replay mismatch;
- cross-tenant evidence collision;
- acceptance of malformed or unauthorised input;
- panic, undefined behaviour, deadlock, or persistent livelock;
- lost required evidence;
- regional failover that changes deterministic standing;
- resource use beyond an admitted bound;
- executable or dependency provenance mismatch;
- security or supply-chain compromise;
- inability to reproduce the candidate from its recorded object graph.

## Rollback procedure

1. Stop new admissions at the deployment boundary.
2. Disable automatic retries that could duplicate irreversible actuation.
3. Preserve in-flight OCEL evidence and execution receipts.
4. Snapshot deployment configuration, runtime identity, and affected execution identifiers.
5. Restore the last admitted release and its exact matching configuration.
6. Verify the restored SHA, artifact digests, verifier identity, and receipt version.
7. Replay or reconcile every interrupted execution according to its typed state.
8. Emit an incident receipt containing the failed candidate, observed fault, affected executions, rollback target, and reconciliation result.
9. Verify post-rollback admission, replay, telemetry, and evidence retention.
10. Reopen promotion only after a repaired candidate completes the full admission ladder.

Rollback never rewrites or discards evidence from the failed release.

## Evidence retention

Retain at minimum:

- production-admission receipt, digest, and evidence root;
- all retained command logs and complete-stream hashes;
- generated-artifact identity results;
- exact repository, toolchain, Action, and dependency SHAs;
- Cargo SBOM precursor files;
- audit, licence, and source-policy results;
- Miri and Clippy results;
- reproducible-build identity results;
- benchmark outputs;
- chaos and stress outputs;
- deployment, promotion, rollback, and incident receipts.

GitHub artifact retention is not a substitute for the target organisation's immutable audit storage and regulatory retention policy.

## Current exclusions

Until separately admitted, this runbook does not claim:

- support for every cloud, operating system, processor, libc, or container runtime;
- compatibility with unverified older receipt, ABI, WIT, or runtime-profile versions;
- unlimited plan size, worker count, duration, throughput, or evidence volume;
- safe actuation through an external path that bypasses BCINR admission and receipts;
- production standing for a candidate whose capability work has not been integrated;
- production standing from release-tool unit validation alone.
