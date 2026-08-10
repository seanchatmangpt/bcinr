//! Integration tests for `src/bin/soundness_cli.rs`: real process invocation
//! of the built binary via `std::process::Command`, real stdin JSON in, real
//! stdout JSON out. No mocking of `check_soundness()` -- these assert on the
//! actual verdict the real `WfNet::check_soundness()` produces for two
//! hand-constructed WF-nets.

use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::Value;

fn run_cli(request: &Value) -> Value {
    let mut child = Command::new(env!("CARGO_BIN_EXE_soundness_cli"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn soundness_cli binary");

    child
        .stdin
        .take()
        .expect("child stdin")
        .write_all(request.to_string().as_bytes())
        .expect("write request to stdin");

    let output = child.wait_with_output().expect("wait for child");
    assert!(
        output.status.success(),
        "soundness_cli exited non-zero; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("stdout is valid JSON")
}

/// A trivial two-step sequential WF-net: p1 -t1-> p2 -t2-> p3.
///
/// Known sound by hand: every transition fires exactly once on the single
/// path from source to sink, so `no_dead_transitions`, `option_to_complete`,
/// and `proper_completion` are all trivially satisfied.
#[test]
fn sequential_net_is_reported_sound() {
    let request = serde_json::json!({
        "places": ["p1", "p2", "p3"],
        "transitions": [
            {"id": "t1", "name": "a"},
            {"id": "t2", "name": "b"}
        ],
        "flow": [
            {"from": "p1", "to": "t1", "kind": "place_to_transition"},
            {"from": "t1", "to": "p2", "kind": "transition_to_place"},
            {"from": "p2", "to": "t2", "kind": "place_to_transition"},
            {"from": "t2", "to": "p3", "kind": "transition_to_place"}
        ],
        "source": "p1",
        "sink": "p3"
    });

    let response = run_cli(&request);

    assert_eq!(response["sound"], Value::Bool(true));
    assert_eq!(response["no_dead_transitions"], Value::Bool(true));
    assert_eq!(response["option_to_complete"], Value::Bool(true));
    assert_eq!(response["proper_completion"], Value::Bool(true));
    assert_eq!(response["is_safe"], Value::Bool(true));
    assert_eq!(response["truncated"], Value::Bool(false));
    assert_eq!(response["reachable_marking_count"], Value::from(3));
}

/// An XOR-split / AND-join mismatch: `t1a` or `t1b` (never both) fires from
/// the source, marking exactly one of `p2`/`p3`. `t2` requires a token in
/// BOTH `p2` and `p3` simultaneously to fire -- which never happens, since
/// the single source token was already consumed by whichever of `t1a`/`t1b`
/// fired. `t2` is therefore a genuine dead transition: structurally it sits
/// on a source->sink path (`p1 -> t1a -> p2 -> t2 -> p4 -> t5 -> p5`), but no
/// reachable marking ever enables it. `t3`/`t4`/`t5` complete the net so it
/// still passes the structural WF-net validity checks in `WfNet::new`
/// (unique source/sink, every node on *some* source->sink path).
///
/// Verified by hand: the only two reachable markings after the initial
/// firing are `[p2: 1]` (via t1a) or `[p3: 1]` (via t1b) -- never both `p2`
/// and `p3` marked together -- so `t2`'s precondition is never satisfied.
#[test]
fn xor_split_and_join_mismatch_has_dead_transition() {
    let request = serde_json::json!({
        "places": ["p1", "p2", "p3", "p4", "p5"],
        "transitions": [
            {"id": "t1a", "name": "choose_a"},
            {"id": "t1b", "name": "choose_b"},
            {"id": "t2", "name": "and_join_dead"},
            {"id": "t3", "name": "finish_a"},
            {"id": "t4", "name": "finish_b"},
            {"id": "t5", "name": "finish_join"}
        ],
        "flow": [
            {"from": "p1", "to": "t1a", "kind": "place_to_transition"},
            {"from": "t1a", "to": "p2", "kind": "transition_to_place"},
            {"from": "p1", "to": "t1b", "kind": "place_to_transition"},
            {"from": "t1b", "to": "p3", "kind": "transition_to_place"},
            {"from": "p2", "to": "t2", "kind": "place_to_transition"},
            {"from": "p3", "to": "t2", "kind": "place_to_transition"},
            {"from": "t2", "to": "p4", "kind": "transition_to_place"},
            {"from": "p2", "to": "t3", "kind": "place_to_transition"},
            {"from": "t3", "to": "p5", "kind": "transition_to_place"},
            {"from": "p3", "to": "t4", "kind": "place_to_transition"},
            {"from": "t4", "to": "p5", "kind": "transition_to_place"},
            {"from": "p4", "to": "t5", "kind": "place_to_transition"},
            {"from": "t5", "to": "p5", "kind": "transition_to_place"}
        ],
        "source": "p1",
        "sink": "p5"
    });

    let response = run_cli(&request);

    assert_eq!(response["sound"], Value::Bool(false));
    assert_eq!(
        response["no_dead_transitions"],
        Value::Bool(false),
        "t2 must be flagged as a dead transition"
    );
    // The other two soundness clauses aren't broken by this construction --
    // named explicitly so a future edit that breaks them doesn't hide behind
    // the already-false `sound` verdict.
    assert_eq!(response["option_to_complete"], Value::Bool(true));
    assert_eq!(response["proper_completion"], Value::Bool(true));
    assert_eq!(response["truncated"], Value::Bool(false));
}
