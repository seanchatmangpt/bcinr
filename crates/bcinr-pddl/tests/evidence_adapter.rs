//! Phase B acceptance: the adapter follows the command, not the narrative.
//!
//! The load-bearing tests here are the ones asserting that *no* fact is emitted.
//! A compile error exits non-zero exactly like a test failure does; treating
//! them alike would make the controller accuse working code of failing.

use std::path::{Path, PathBuf};

use bcinr_pddl::evidence::{
    observation_from, parse_libtest_json, CommandRun, EvidenceFact, EvidenceLedger, LedgerError,
    Observation, SuiteOutcome, UnobservableReason,
};

/// A real passing suite, captured verbatim from
/// `cargo test -p bcinr-pddl --test typed_grounding -- --format json -Z unstable-options`.
const PASSING_STDOUT: &str = r#"{ "type": "suite", "event": "started", "test_count": 1 }
{ "type": "test", "event": "started", "name": "typed_grounding_restricts_to_type_compatible_bindings" }
{ "type": "test", "name": "typed_grounding_restricts_to_type_compatible_bindings", "event": "ok" }
{ "type": "suite", "event": "ok", "passed": 1, "failed": 0, "ignored": 0, "measured": 0, "filtered_out": 0, "exec_time": 0.007714625 }
"#;

const FAILING_STDOUT: &str = r#"{ "type": "suite", "event": "started", "test_count": 2 }
{ "type": "test", "event": "started", "name": "alpha" }
{ "type": "test", "name": "alpha", "event": "ok" }
{ "type": "test", "event": "started", "name": "beta" }
{ "type": "test", "name": "beta", "event": "failed", "stdout": "assertion failed: 1 == 2\n" }
{ "type": "suite", "event": "failed", "passed": 1, "failed": 1, "ignored": 0, "measured": 0, "filtered_out": 0, "exec_time": 0.01 }
"#;

/// A slow suite. libtest emits an informational `timeout` event for any test
/// running over 60 s, and then emits that test's real result afterwards. This is
/// the shape that a naive "any non-ok event means failure" rule misreads.
const SLOW_BUT_PASSING_STDOUT: &str = r#"{ "type": "suite", "event": "started", "test_count": 1 }
{ "type": "test", "event": "started", "name": "full_horizon_does_not_reach_the_whole_release" }
{ "type": "test", "event": "timeout", "name": "full_horizon_does_not_reach_the_whole_release" }
{ "type": "test", "name": "full_horizon_does_not_reach_the_whole_release", "event": "ok" }
{ "type": "suite", "event": "ok", "passed": 1, "failed": 0, "ignored": 0, "measured": 0, "filtered_out": 0, "exec_time": 64.64 }
"#;

fn run(stdout: &str, exit: Option<i32>) -> CommandRun {
    CommandRun {
        argv: vec!["cargo".into(), "test".into()],
        cwd: PathBuf::from("/repo"),
        exit_status: exit,
        stdout: stdout.to_string(),
        stderr: String::new(),
        duration_ms: 1,
    }
}

fn observe(stdout: &str, exit: Option<i32>) -> (SuiteOutcome, Option<Observation>) {
    let command = run(stdout, exit);
    let outcome = parse_libtest_json(&command.stdout, command.exit_status);
    observation_from("baseline-repair", "baseline-nonvacuity", command, outcome)
}

#[test]
fn passing_suite_emits_test_pass() {
    let (outcome, observation) = observe(PASSING_STDOUT, Some(0));

    assert_eq!(
        outcome,
        SuiteOutcome::Passed {
            passed: 1,
            ignored: 0
        }
    );
    let observation = observation.expect("a passing suite must produce a fact");
    assert_eq!(
        observation.fact,
        EvidenceFact::TestPass {
            phase: "baseline-repair".into(),
            suite: "baseline-nonvacuity".into()
        }
    );
    assert_eq!(
        observation.fact.render_atom(),
        "(test-passed baseline-repair baseline-nonvacuity)"
    );
}

#[test]
fn failing_suite_emits_test_fail() {
    let (outcome, observation) = observe(FAILING_STDOUT, Some(101));

    assert_eq!(
        outcome,
        SuiteOutcome::Failed {
            passed: 1,
            failed: 1
        }
    );
    let observation = observation.expect("a failing suite must produce a fact");
    assert_eq!(
        observation.fact.render_atom(),
        "(observed-test-fail baseline-repair baseline-nonvacuity)"
    );
}

/// The single most important test in this file.
///
/// A compile error exits non-zero having never run a test, so there is no
/// `suite` event. If the adapter treated non-zero exit as failure, a broken
/// build would assert that the code under test is wrong.
#[test]
fn compile_error_emits_no_fact_at_all() {
    let stderr_only = ""; // cargo wrote the error to stderr; stdout has no JSON
    let (outcome, observation) = observe(stderr_only, Some(101));

    assert_eq!(
        outcome,
        SuiteOutcome::Unobservable {
            reason: UnobservableReason::NoSuiteEvent
        }
    );
    assert!(
        observation.is_none(),
        "a build failure must emit NO fact -- absence of evidence is not \
         evidence of failure"
    );
}

#[test]
fn signal_death_emits_no_fact_and_is_distinguishable_from_a_bad_exit() {
    let (outcome, observation) = observe("", None);

    assert_eq!(
        outcome,
        SuiteOutcome::Unobservable {
            reason: UnobservableReason::KilledBySignal
        },
        "a signal death must be distinguishable from a non-zero exit"
    );
    assert!(observation.is_none());
}

/// A slow test must not be read as a failed test.
#[test]
fn informational_timeout_event_does_not_fabricate_a_failure() {
    let (outcome, observation) = observe(SLOW_BUT_PASSING_STDOUT, Some(0));

    assert_eq!(
        outcome,
        SuiteOutcome::Passed {
            passed: 1,
            ignored: 0
        },
        "libtest's informational `timeout` event must not become a failure"
    );
    assert!(matches!(
        observation.map(|o| o.fact),
        Some(EvidenceFact::TestPass { .. })
    ));
}

/// When the exit status and the suite event disagree, neither is admitted.
#[test]
fn status_contradicting_the_suite_emits_no_fact() {
    // Suite says ok, process exited non-zero -- something else broke.
    let (outcome, observation) = observe(PASSING_STDOUT, Some(1));
    assert!(
        matches!(
            outcome,
            SuiteOutcome::Unobservable {
                reason: UnobservableReason::StatusContradictsSuite { .. }
            }
        ),
        "got {outcome:?}"
    );
    assert!(observation.is_none());

    // Suite says failed, process exited zero -- equally contradictory.
    let (outcome, observation) = observe(FAILING_STDOUT, Some(0));
    assert!(matches!(
        outcome,
        SuiteOutcome::Unobservable {
            reason: UnobservableReason::StatusContradictsSuite { .. }
        }
    ));
    assert!(observation.is_none());
}

/// The scenario the previous workflow actually produced: a model-authored
/// summary claiming success while the command shows failure.
///
/// The adapter never reads the narrative, so this reduces to "follow the
/// command" -- but the test pins it explicitly because it is the whole premise.
#[test]
fn model_authored_claim_of_success_cannot_override_a_failing_command() {
    // Verbatim shape from a real workflow journal.jsonl `result` object.
    let narrative = r#"{"total_passing":730,"total_failing":0,"all_tests_passing":true}"#;

    let (outcome, observation) = observe(FAILING_STDOUT, Some(101));

    assert_eq!(
        outcome,
        SuiteOutcome::Failed {
            passed: 1,
            failed: 1
        }
    );
    let observation = observation.expect("the command failed, so a fact is owed");
    assert!(
        matches!(observation.fact, EvidenceFact::TestFail { .. }),
        "the command failed; a narrative claiming {narrative} must not change that"
    );
}

#[test]
fn ledger_chains_and_verifies() {
    let mut ledger = EvidenceLedger::new();
    assert!(ledger.is_empty());
    ledger.verify().expect("an empty ledger verifies");

    for suite in ["alpha", "beta", "gamma"] {
        let command = run(PASSING_STDOUT, Some(0));
        let outcome = parse_libtest_json(&command.stdout, command.exit_status);
        let (_, observation) = observation_from("baseline-repair", suite, command, outcome);
        ledger.append(observation.expect("passing suite yields an observation"));
    }

    assert_eq!(ledger.len(), 3);
    ledger.verify().expect("a well-formed ledger verifies");

    let atoms = ledger.admitted_atoms();
    assert_eq!(atoms.len(), 3);
    assert!(atoms[0].starts_with("(test-passed baseline-repair alpha"));

    // Round-trip through JSON.
    let json = ledger.to_json().expect("serializes");
    let restored = EvidenceLedger::from_json(&json).expect("deserializes");
    assert_eq!(restored, ledger);
    restored.verify().expect("a round-tripped ledger verifies");
}

/// A ledger whose seal cannot be checked is a claim, not evidence. These are the
/// three tamperings that must each be caught.
#[test]
fn ledger_verify_rejects_tampering() {
    let mut ledger = EvidenceLedger::new();
    for suite in ["alpha", "beta"] {
        let command = run(PASSING_STDOUT, Some(0));
        let outcome = parse_libtest_json(&command.stdout, command.exit_status);
        let (_, observation) = observation_from("baseline-repair", suite, command, outcome);
        ledger.append(observation.expect("observation"));
    }
    ledger.verify().expect("baseline verifies");

    // 1. Rewrite a recorded fact from pass to fail, keeping the root.
    let mut forged = ledger.clone();
    forged.entries[0].observation.fact = EvidenceFact::TestFail {
        phase: "baseline-repair".into(),
        suite: "alpha".into(),
    };
    assert!(
        matches!(
            forged.verify(),
            Err(LedgerError::RootMismatch { sequence: 0, .. })
        ),
        "rewriting an admitted fact must break the chain, got {:?}",
        forged.verify()
    );

    // 2. Rewrite the outcome while leaving the fact alone.
    let mut forged = ledger.clone();
    forged.entries[1].observation.outcome = SuiteOutcome::Failed {
        passed: 0,
        failed: 9,
    };
    assert!(
        matches!(
            forged.verify(),
            Err(LedgerError::RootMismatch { sequence: 1, .. })
        ),
        "the outcome is part of the commitment"
    );

    // 3. Drop an entry, which is what a convenient deletion looks like.
    let mut forged = ledger.clone();
    forged.entries.remove(0);
    assert!(
        forged.verify().is_err(),
        "deleting an entry must not verify"
    );
}

/// Post-seal append must be detectable. `OcelLog` cannot do this -- its
/// `seal_receipt(&self)` leaves the log mutable and nothing recomputes the
/// digest. Here the head root moves, so an appended entry is visible.
#[test]
fn appending_after_reading_the_root_changes_the_root() {
    let mut ledger = EvidenceLedger::new();
    let command = run(PASSING_STDOUT, Some(0));
    let outcome = parse_libtest_json(&command.stdout, command.exit_status);
    let (_, observation) = observation_from("baseline-repair", "alpha", command, outcome);
    ledger.append(observation.expect("observation"));

    let sealed_root = ledger.root.clone();

    let command = run(PASSING_STDOUT, Some(0));
    let outcome = parse_libtest_json(&command.stdout, command.exit_status);
    let (_, observation) = observation_from("baseline-repair", "beta", command, outcome);
    ledger.append(observation.expect("observation"));

    assert_ne!(
        sealed_root, ledger.root,
        "an append after sealing must change the head root"
    );
    ledger
        .verify()
        .expect("the extended ledger is still internally consistent");
}

/// End-to-end against a real cargo invocation, so the parser is exercised
/// against output this session actually produced rather than a fixture.
#[test]
fn end_to_end_against_a_real_cargo_run() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root")
        .to_path_buf();

    let argv: Vec<String> = [
        "cargo",
        "test",
        "-p",
        "bcinr-pddl",
        "--test",
        "typed_grounding",
        "--",
        "--format",
        "json",
        "-Z",
        "unstable-options",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    let command = CommandRun::execute(&argv, &repo_root).expect("cargo runs");
    let outcome = parse_libtest_json(&command.stdout, command.exit_status);

    assert!(
        matches!(outcome, SuiteOutcome::Passed { .. }),
        "typed_grounding is a passing suite; adapter said {outcome:?}\nstderr:\n{}",
        command.stderr
    );
    assert!(command.succeeded());
    assert!(
        !command.digest().is_empty() && command.digest().len() == 64,
        "digest must be a 64-hex BLAKE3"
    );
}
