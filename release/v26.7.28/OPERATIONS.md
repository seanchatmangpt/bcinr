# BCINR v26.7.28 Production Operations

## Release boundary

This runbook governs promotion of the exact BCINR v26.7.28 release candidate. It does not grant standing to a source branch, local checkout, benchmark report, generated document, or previously verified commit.

The promoted object is one exact commit SHA with one exact ggen manufacturer SHA and one complete production-admission receipt.

## Preconditions

Promotion is refused unless:

- the production-admission workflow completed against the exact candidate SHA;
- the receipt standing is `ALIVE`;
- the receipt BLAKE3 digest matches the retained receipt;
- every required rail exited successfully;
- the ggen outputs are byte-identical to the BCINR consumed outputs;
- the candidate is mergeable with current `main`;
- there are no unresolved P0 or P1 defects;
- deployment and rollback owners have acknowledged the exact candidate;
- the deployment target supports every capability claimed for that target.

A skipped, cancelled, stale, or unavailable gate is not a passing gate.

## Promotion sequence

### 1. Exact candidate admission

Record:

- BCINR candidate SHA;
- ggen manufacturer SHA;
- `Cargo.lock` digest;
- release-profile digest;
- production receipt digest;
- target environment identifier;
- deployment owner;
- rollback owner.

### 2. Controlled canary

Deploy the exact candidate to a tenant-isolated canary with irreversible external actuation disabled or routed to a controlled sink.

Exercise:

- deterministic execution;
- deterministic replay;
- typed refusal;
- evidence export;
- restart recovery;
- duplicate-delivery handling;
- bounded resource exhaustion;
- operational telemetry and alert routing.

Promotion stops on any receipt mismatch, duplicate actuation, panic, deadlock, evidence loss, unbounded resource growth, or unexplained latency regression.

### 3. Regional production canary

Enable bounded production traffic in one region and one tenant class. Preserve a direct rollback path to the previously admitted release.

Compare:

- admitted requests;
- refused requests by refusal code;
- execution and replay roots;
- resource saturation;
- process restarts;
- error budgets;
- latency and throughput against the admitted baseline.

### 4. Multi-region expansion

Expand only after the regional canary produces complete receipts and no release blocker. Validate active-active ordering, regional failover, idempotent retry, and tenant evidence isolation before global promotion.

### 5. Global promotion

Promote the same bytes and configuration that passed canary. Rebuilding or regenerating artifacts creates a new candidate and requires a new admission receipt.

## Rollback triggers

Rollback is mandatory for:

- unreceipted actuation;
- duplicate irreversible actuation;
- receipt or replay mismatch;
- cross-tenant evidence collision;
- acceptance of malformed or unauthorised input;
- panic, undefined behavior, deadlock, or persistent livelock;
- lost or truncated required evidence;
- regional failover that changes deterministic standing;
- resource use beyond an admitted bound;
- security or supply-chain compromise;
- inability to reproduce the candidate from its recorded object graph.

## Rollback procedure

1. Stop new admissions at the deployment boundary.
2. Preserve in-flight OCEL evidence and execution receipts.
3. Prevent automatic retry from producing duplicate actuation.
4. Restore the last admitted release and its matching configuration.
5. Verify the restored release SHA and artifact digests.
6. Replay or reconcile every interrupted execution according to its typed state.
7. Emit an incident receipt containing the failed candidate, observed fault, affected executions, rollback target, and reconciliation outcome.
8. Reopen promotion only after a repaired candidate completes the full admission ladder.

Rollback does not rewrite or discard evidence from the failed release.

## Evidence retention

Retain at minimum:

- production-admission receipt and BLAKE3 digest;
- all command logs;
- generated-artifact identity results;
- exact repository and dependency SHAs;
- benchmark outputs;
- chaos and stress outputs;
- SBOM and security reports when added to the profile;
- deployment and rollback receipts;
- incident evidence for any refused promotion.

Retention policy must satisfy the target organisation's audit and regulatory requirements. The default GitHub workflow retention is not a substitute for enterprise evidence retention.

## Current exclusions

Until separately admitted, this runbook does not claim:

- every cloud, operating system, processor, or container runtime is supported;
- compatibility with unverified older receipt or ABI versions;
- unlimited plan size, worker count, duration, throughput, or evidence volume;
- safe direct actuation into an external system that bypasses BCINR admission and receipt controls;
- production standing for a candidate whose capability PRs have not been integrated.
