//! Integration tests for `src/bin/cmca_allocate_cli.rs`: real process
//! invocation of the built binary via `std::process::Command`, real stdin
//! JSON in, real stdout JSON out. No mocking of `allocate()` -- these assert
//! on the actual deterministic allocation the real `allocator::allocate()`
//! produces for the `case_studies` fixture, untampered and tampered.

use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::Value;

fn run_cli(request: &Value) -> Value {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cmca_allocate_cli"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn cmca_allocate_cli binary");

    child
        .stdin
        .take()
        .expect("child stdin")
        .write_all(request.to_string().as_bytes())
        .expect("write request to stdin");

    let output = child.wait_with_output().expect("wait for child");
    assert!(
        output.status.success(),
        "cmca_allocate_cli exited non-zero; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("stdout is valid JSON")
}

#[test]
fn untampered_case_reports_identical_reference_and_claimed_allocations() {
    let request = serde_json::json!({ "case": "case_studies", "tamper": null });
    let response = run_cli(&request);

    assert_eq!(response["case"], Value::from("case_studies"));
    assert_eq!(response["tampered"], Value::Bool(false));
    assert_eq!(
        response["reference_allocation"], response["claimed_allocation"],
        "untampered request must report identical reference and claimed allocations"
    );

    // Sanity check on the real allocator output for this fixture: cache
    // choice case study 1's known property (Artifact_A > Artifact_B), same
    // as `tests/case_studies.rs`'s `test_case_study_1_cache_choice`.
    let obj_0 = response["reference_allocation"]["0"].as_f64().unwrap();
    let obj_1 = response["reference_allocation"]["1"].as_f64().unwrap();
    assert!(obj_0.is_finite() && obj_0 >= 0.0);
    assert!(obj_1.is_finite() && obj_1 >= 0.0);
    assert!(
        obj_0 > obj_1,
        "Artifact_A (index 0) should have higher cache allocation than Artifact_B (index 1)"
    );
}

#[test]
fn tampered_case_diverges_only_at_the_tampered_index() {
    let request = serde_json::json!({
        "case": "case_studies",
        "tamper": { "index": 0, "delta_millionths": 300000 }
    });
    let response = run_cli(&request);

    assert_eq!(response["tampered"], Value::Bool(true));

    let reference = response["reference_allocation"].as_object().unwrap();
    let claimed = response["claimed_allocation"].as_object().unwrap();

    assert_ne!(
        reference, claimed,
        "tampered request must produce a claimed allocation that differs from the reference"
    );

    let ref_0 = reference["0"].as_f64().unwrap();
    let claimed_0 = claimed["0"].as_f64().unwrap();
    assert!(
        (claimed_0 - (ref_0 + 0.3)).abs() < 1e-9,
        "claimed[0] should equal reference[0] + 0.3, got claimed={claimed_0} reference={ref_0}"
    );

    for key in reference.keys() {
        if key == "0" {
            continue;
        }
        assert_eq!(
            reference[key], claimed[key],
            "only the tampered index should diverge between reference and claimed"
        );
    }
}

#[test]
fn unknown_case_is_refused_with_error_envelope() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cmca_allocate_cli"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn cmca_allocate_cli binary");

    child
        .stdin
        .take()
        .expect("child stdin")
        .write_all(br#"{"case":"not_a_real_case","tamper":null}"#)
        .expect("write request to stdin");

    let output = child.wait_with_output().expect("wait for child");
    assert!(
        !output.status.success(),
        "unknown case should exit non-zero"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error"),
        "stderr should contain an error envelope, got: {stderr}"
    );
}

// ─── Chicago-style hardening: the production surface must refuse, never
// panic, on every hostile input shape. Each case spawns the REAL binary.

/// Spawns the binary, feeds raw stdin bytes, returns (exit_ok, stderr).
fn run_cli_raw(stdin_bytes: &[u8]) -> (bool, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cmca_allocate_cli"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn cmca_allocate_cli binary");
    child
        .stdin
        .take()
        .expect("child stdin")
        .write_all(stdin_bytes)
        .expect("write raw stdin");
    let output = child.wait_with_output().expect("wait for child");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

/// Every hostile shape must exit non-zero with a JSON error envelope on
/// stderr -- never a Rust panic, never exit 0 with garbage.
#[test]
fn malformed_and_hostile_inputs_are_typed_refusals_never_panics() {
    let hostile: &[(&str, &[u8])] = &[
        ("empty stdin", b""),
        ("invalid JSON syntax", b"{not json"),
        ("truncated JSON", br#"{"case":"case_studies""#),
        ("null body", b"null"),
        ("array body", b"[1,2,3]"),
        ("case is a number", br#"{"case":123}"#),
        ("case missing", br#"{"tamper":null}"#),
        (
            "tamper is a string",
            br#"{"case":"case_studies","tamper":"x"}"#,
        ),
        (
            "tamper missing index",
            br#"{"case":"case_studies","tamper":{"delta_millionths":1}}"#,
        ),
        (
            "tamper delta is a string",
            br#"{"case":"case_studies","tamper":{"index":0,"delta_millionths":"1"}}"#,
        ),
        (
            "tamper index negative (serde usize refusal)",
            br#"{"case":"case_studies","tamper":{"index":-1,"delta_millionths":1}}"#,
        ),
        (
            "tamper index float",
            br#"{"case":"case_studies","tamper":{"index":0.5,"delta_millionths":1}}"#,
        ),
    ];
    for (name, bytes) in hostile {
        let (ok, stderr) = run_cli_raw(bytes);
        assert!(!ok, "{name}: must exit non-zero, got success");
        assert!(
            stderr.contains("\"error\""),
            "{name}: stderr must be a JSON error envelope, got: {stderr}"
        );
        assert!(
            !stderr.contains("panicked"),
            "{name}: the binary must never panic, got: {stderr}"
        );
    }
}

/// Out-of-range tamper indices -- including usize::MAX -- are typed
/// refusals naming the range, not index panics.
#[test]
fn tamper_index_out_of_range_is_a_typed_refusal_at_every_boundary() {
    for index in [8usize, 9, 100, usize::MAX] {
        let request = format!(
            r#"{{"case":"case_studies","tamper":{{"index":{index},"delta_millionths":1}}}}"#
        );
        let (ok, stderr) = run_cli_raw(request.as_bytes());
        assert!(!ok, "tamper index {index} must exit non-zero");
        assert!(
            stderr.contains("out of range"),
            "tamper index {index} refusal should name the range, got: {stderr}"
        );
        assert!(
            !stderr.contains("panicked"),
            "tamper index {index} must not panic, got: {stderr}"
        );
    }
}

/// Boundary of the VALID domain: the last object index (N-1 = 7) tampered
/// with the most extreme deltas still produces a well-formed response
/// where only index 7 diverges.
#[test]
fn tamper_at_the_last_valid_index_with_extreme_deltas_diverges_only_there() {
    for delta_millionths in [i64::MAX, i64::MIN + 1, 1, -1] {
        let request = serde_json::json!({
            "case": "case_studies",
            "tamper": { "index": 7, "delta_millionths": delta_millionths }
        });
        let response = run_cli(&request);
        assert_eq!(response["tampered"], Value::Bool(true));
        let reference = response["reference_allocation"].as_object().unwrap();
        let claimed = response["claimed_allocation"].as_object().unwrap();
        for key in reference.keys() {
            if key == "7" {
                assert_ne!(
                    reference[key], claimed[key],
                    "index 7 must diverge at delta {delta_millionths}"
                );
            } else {
                assert_eq!(
                    reference[key], claimed[key],
                    "only index 7 may diverge (delta {delta_millionths})"
                );
            }
        }
    }
}
