# Chicago TDD validation and CMCA JTBD extension — v26.7.25

## Scope

This receipt validates `crates/bcinr-powl/tests/chicago_tdd_integration.rs` and adds a bounded Chicago-TDD suite over the admitted CMCA job path:

```text
MeasurementArtifact
  -> evaluate_calibration
  -> CertificateReceipt
  -> AdaptiveUpdate admission
  -> allocate
```

## Observed execution

The reported test file and exact command are real. The original four-test binary passed both in the PR #14 audit replay and again in focused workflow run `30185400795` under:

```bash
cargo test -p bcinr-powl --test chicago_tdd_integration --features std
```

Standing: **ALIVE — 4 passed, 0 failed**.

Execution success does not establish every stronger semantic claim made by the report.

## Validation findings

| Claim | Standing | Evidence boundary |
|---|---|---|
| The four named tests execute and pass | ALIVE | Reproduced with the report's exact command on Linux/nightly Rust. |
| A dependent operation cannot fire before its predecessor | UNSUPPORTED | The validator accumulates every `op_fired` bit before checking predecessor membership in the final trace. `op1; op0; seal(0b11)` is therefore not rejected for temporal inversion. |
| The sealed Chicago receipt contains the POWL operation trace | UNSUPPORTED | `OcelCollector` receives a conformance diagnostic; the separate `bcinr_powl::ocel::OcelLog` contains operation events. |
| The export is fully OCEL 2.0 compliant | PARTIAL_ALIVE | The test proves JSON parsing, required key presence, event/object type names, timestamps, and relationship closure. It does not run an independent schema validator or process-mining importer. |
| XOR executes exactly one branch | UNSUPPORTED | The test runs an XOR workflow but never asserts the number or identity of fired branch operations. Its assertions begin at exported JSON structure. |
| Receipt digests are deterministic for identical event content | UNSUPPORTED | The test explicitly avoids equality because generated event UUIDs may differ. |
| Different collector contents produce different observed digests | ALIVE | Different observed collectors produce different digests. The test does not isolate each field mutation independently. |

## Added CMCA Chicago-TDD scenarios

`crates/bcinr-cmca/tests/jtbd_certified_actuation_chicago.rs` uses real production collaborators and state-based assertions:

1. material, stable telemetry issues a control-mode receipt, admits an adaptive update, and completes allocation;
2. drifting telemetry is typed-refused, then the allocator degrades to selection-only without mutable-state drift;
3. an Observatory certificate cannot authorize a job whose state, envelope, or outcome receipt has a different digest;
4. an invalid allocator certificate returns `CertificateDigestMismatch` and preserves weights, switch time, and mode;
5. insufficient dwell returns `ModeDwellTimeViolated` and preserves the same persistent state.

Standing: **ALIVE — 5 passed, 0 failed**.

## Verification receipt

Workflow run `30185400795` completed every admitted step successfully:

```bash
cargo test -p bcinr-cmca --test jtbd_certified_actuation_chicago
cargo test -p bcinr-cmca
cargo test -p bcinr-powl --test chicago_tdd_integration --features std
cargo fmt --all -- --check
cargo clippy -p bcinr-cmca --test jtbd_certified_actuation_chicago -- -D warnings
```

The temporary verifier self-deleted after committing `docs/testing/chicago-tdd-cmca-jtbd-execution-v26.7.25.md`.

## Exclusions

This change does not repair the POWL temporal-predecessor validator, claim independent OCEL schema conformance, or add recovery-only CMCA authority modules that are not admitted on the active repository base. Those are separate bounded changes with separate falsifiers.
