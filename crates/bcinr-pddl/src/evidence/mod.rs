//! Evidence adapter: real verifier output becomes admitted planning facts.
//!
//! The controller may only admit a fact that a command's own output supports.
//! Nothing here reads a model-authored summary, a workflow `journal.jsonl`
//! `result` object, or a phase report's prose. Those are narrative; this module
//! deals only in what a process actually printed and what status it actually
//! exited with.
//!
//! # The distinction that matters
//!
//! A broken build and a failing test both exit non-zero, but only the latter
//! emits a libtest `suite` event. Conflating them would let a compile error
//! assert `observed-test-fail`, which is a false accusation against the code
//! under test. So the outcome type is three-valued, not a boolean:
//!
//! ```text
//! exit 0   + suite event, failed == 0   -> Passed       -> observed-test-pass
//! exit != 0 + suite event, failed > 0   -> Failed       -> observed-test-fail
//! anything else                         -> Unobservable -> NO FACT
//! ```
//!
//! Absence of evidence is not evidence of failure. An `Unobservable` run leaves
//! the phase exactly where it was, and the controller replans from unchanged
//! state rather than inventing a verdict.
//!
//! # Why the suite event and not per-test events
//!
//! libtest emits `{"type":"test","event":"timeout",...}` for any test running
//! over 60 seconds, and then emits that same test's real `ok` result later. A
//! rule like "any non-ok event means failure" fabricates failures for slow
//! tests. Keying the verdict on the terminal `suite` event avoids this by
//! construction — per-test events never decide anything.
//!
//! # Dependencies
//!
//! Deliberately none beyond `blake3` and `serde`, both unconditional
//! dependencies of this crate. In particular this module does not use
//! `problem_builder` or `bcinr-powl`'s `receipt::chain`, which are both gated
//! behind the optional `mfw-planner` feature and so unavailable by default.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use serde::{Deserialize, Serialize};

mod ledger;
pub use ledger::{EvidenceLedger, LedgerEntry, LedgerError};

/// Domain-separation prefix, matching the house style used by `digest_tape`
/// and friends in `bcinr-powl`'s `receipt` module.
const EVIDENCE_DOMAIN: &[u8] = b"bcinr:pddl:evidence:v1";

/// Hash arbitrary bytes under this module's domain separator.
pub fn digest_bytes(bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(EVIDENCE_DOMAIN);
    hasher.update(&(bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

/// One actually-executed command, with its status genuinely captured.
///
/// Every existing `Command` call site in this workspace discards
/// `Output::status`; this type exists so that this one does not.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandRun {
    pub argv: Vec<String>,
    pub cwd: PathBuf,
    /// `None` means the process was terminated by a signal and produced no
    /// exit code. That is distinct from a non-zero exit and is never a verdict.
    pub exit_status: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u128,
}

impl CommandRun {
    /// Run a command, capturing stdout and stderr *separately*.
    ///
    /// Keeping the streams apart is required, not stylistic: libtest writes its
    /// JSON to stdout while cargo writes build warnings to stderr, so merging
    /// them (`2>&1`) corrupts the JSON stream.
    pub fn execute(argv: &[String], cwd: &Path) -> std::io::Result<Self> {
        let (program, args) = argv
            .split_first()
            .ok_or_else(|| std::io::Error::other("empty argv"))?;

        let started = Instant::now();
        let output = Command::new(program).args(args).current_dir(cwd).output()?;
        let duration_ms = started.elapsed().as_millis();

        Ok(Self {
            argv: argv.to_vec(),
            cwd: cwd.to_path_buf(),
            exit_status: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            duration_ms,
        })
    }

    pub fn succeeded(&self) -> bool {
        self.exit_status == Some(0)
    }

    /// Digest over the command identity and both output streams.
    pub fn digest(&self) -> String {
        let mut buf = Vec::new();
        for arg in &self.argv {
            buf.extend_from_slice(arg.as_bytes());
            buf.push(0);
        }
        buf.extend_from_slice(self.cwd.as_os_str().as_encoded_bytes());
        buf.push(0);
        buf.extend_from_slice(&self.exit_status.unwrap_or(i32::MIN).to_le_bytes());
        buf.extend_from_slice(self.stdout.as_bytes());
        buf.push(0);
        buf.extend_from_slice(self.stderr.as_bytes());
        digest_bytes(&buf)
    }
}

/// Why a run produced no usable verdict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnobservableReason {
    /// No libtest `suite` event was present. The usual cause is a compile
    /// error: cargo exits non-zero having never run a test. This must never
    /// become `observed-test-fail`.
    NoSuiteEvent,
    /// The process was killed by a signal, so there is no exit code at all.
    KilledBySignal,
    /// A `suite` event was found but carried an unrecognised `event` value.
    MalformedSuiteEvent { event: String },
    /// The suite reported success but the process exited non-zero, or vice
    /// versa. The two sources disagree, so neither is admitted.
    StatusContradictsSuite {
        exit_status: Option<i32>,
        suite_event: String,
    },
}

/// The verdict of one verifier run. Three-valued on purpose.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuiteOutcome {
    Passed { passed: u64, ignored: u64 },
    Failed { passed: u64, failed: u64 },
    Unobservable { reason: UnobservableReason },
}

impl SuiteOutcome {
    pub fn is_observable(&self) -> bool {
        !matches!(self, Self::Unobservable { .. })
    }
}

/// Parse a libtest JSON stream into a verdict.
///
/// Only the terminal `{"type":"suite", ...}` event decides the outcome. Per-test
/// events, including the informational `"timeout"` event libtest emits for slow
/// tests, are deliberately ignored.
pub fn parse_libtest_json(stdout: &str, exit_status: Option<i32>) -> SuiteOutcome {
    let mut terminal: Option<serde_json::Value> = None;

    for line in stdout.lines() {
        let line = line.trim();
        if !line.starts_with('{') {
            continue;
        }
        let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        // Key by field name, never by position: libtest emits {type,event,name}
        // on `started` and {type,name,event} on results.
        if value.get("type").and_then(|t| t.as_str()) != Some("suite") {
            continue;
        }
        if value.get("event").and_then(|e| e.as_str()) == Some("started") {
            continue;
        }
        terminal = Some(value);
    }

    let Some(suite) = terminal else {
        // No `suite` event at all. The usual cause is a compile error: cargo
        // exits non-zero having never run a test. Falsified this session by
        // replacing this arm with `Failed`, which correctly broke
        // `compile_error_emits_no_fact_at_all` and
        // `signal_death_emits_no_fact_and_is_distinguishable_from_a_bad_exit`,
        // and nothing else.
        return SuiteOutcome::Unobservable {
            reason: if exit_status.is_none() {
                UnobservableReason::KilledBySignal
            } else {
                UnobservableReason::NoSuiteEvent
            },
        };
    };

    let event = suite
        .get("event")
        .and_then(|e| e.as_str())
        .unwrap_or_default()
        .to_string();
    let field = |k: &str| suite.get(k).and_then(|v| v.as_u64()).unwrap_or(0);
    let (passed, failed, ignored) = (field("passed"), field("failed"), field("ignored"));

    match event.as_str() {
        "ok" if exit_status == Some(0) && failed == 0 => SuiteOutcome::Passed { passed, ignored },
        "failed" if exit_status != Some(0) && failed > 0 => SuiteOutcome::Failed { passed, failed },
        "ok" | "failed" => SuiteOutcome::Unobservable {
            reason: UnobservableReason::StatusContradictsSuite {
                exit_status,
                suite_event: event,
            },
        },
        other => SuiteOutcome::Unobservable {
            reason: UnobservableReason::MalformedSuiteEvent {
                event: other.to_string(),
            },
        },
    }
}

/// A planning fact the controller may admit. There is no variant for "probably
/// passed" or "assumed complete" — only what a command demonstrated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceFact {
    TestPass { phase: String, suite: String },
    TestFail { phase: String, suite: String },
}

impl EvidenceFact {
    /// Render as a ground PDDL atom for the controller problem's `:init`.
    pub fn render_atom(&self) -> String {
        match self {
            Self::TestPass { phase, suite } => format!("(test-passed {phase} {suite})"),
            Self::TestFail { phase, suite } => format!("(observed-test-fail {phase} {suite})"),
        }
    }
}

/// A fact together with everything needed to re-derive it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    pub fact: EvidenceFact,
    pub outcome: SuiteOutcome,
    pub command: CommandRun,
    pub command_digest: String,
}

/// Run a verifier suite and derive whatever fact its output actually supports.
///
/// Returns `Ok(None)` when the run was unobservable — the caller must leave the
/// admitted state unchanged rather than recording anything.
pub fn observe_suite(
    phase: &str,
    suite: &str,
    argv: &[String],
    cwd: &Path,
) -> std::io::Result<(SuiteOutcome, Option<Observation>)> {
    let command = CommandRun::execute(argv, cwd)?;
    let outcome = parse_libtest_json(&command.stdout, command.exit_status);
    Ok(observation_from(phase, suite, command, outcome))
}

/// The pure half of `observe_suite`, so the mapping is testable without
/// spawning a process.
pub fn observation_from(
    phase: &str,
    suite: &str,
    command: CommandRun,
    outcome: SuiteOutcome,
) -> (SuiteOutcome, Option<Observation>) {
    let fact = match &outcome {
        SuiteOutcome::Passed { .. } => Some(EvidenceFact::TestPass {
            phase: phase.to_string(),
            suite: suite.to_string(),
        }),
        SuiteOutcome::Failed { .. } => Some(EvidenceFact::TestFail {
            phase: phase.to_string(),
            suite: suite.to_string(),
        }),
        // The whole point: no fact at all.
        SuiteOutcome::Unobservable { .. } => None,
    };

    let observation = fact.map(|fact| {
        let command_digest = command.digest();
        Observation {
            fact,
            outcome: outcome.clone(),
            command: command.clone(),
            command_digest,
        }
    });

    (outcome, observation)
}
