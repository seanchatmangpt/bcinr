//! Small stdin/stdout JSON CLI wrapping the real `WfNet::check_soundness()`
//! (see `src/wf_net.rs`), so out-of-process callers (e.g. the `autofde-lab`
//! Python bridge) can invoke exhaustive-BFS WF-net soundness checking without
//! reimplementing it.
//!
//! Wire schema is the external contract shared with the Python-side agent
//! building in parallel; see the request/response structs below and the
//! doc comments on any field where this CLI had to make a naming decision
//! not spelled out in the shared schema.

use std::io::{self, Read, Write};

use bcinr_powl::wf_net::WfNet;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct TransitionSpec {
    id: String,
    /// The shared schema says `"name"`, not `"label"` -- `WfNet`'s internal
    /// `Label` type is `Option<String>` (`None` = silent/tau transition).
    /// This CLI treats an absent `name` field as `null` (tau, i.e. no
    /// activity label), matching `WfNet`'s own `Label` semantics rather than
    /// inventing a third state.
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ArcKind {
    PlaceToTransition,
    TransitionToPlace,
}

#[derive(Debug, Deserialize)]
struct ArcSpec {
    from: String,
    to: String,
    kind: ArcKind,
}

#[derive(Debug, Deserialize)]
struct SoundnessRequest {
    places: Vec<String>,
    transitions: Vec<TransitionSpec>,
    flow: Vec<ArcSpec>,
    source: String,
    sink: String,
}

#[derive(Debug, Serialize)]
struct SoundnessResponse {
    no_dead_transitions: Option<bool>,
    option_to_complete: Option<bool>,
    proper_completion: Option<bool>,
    is_safe: Option<bool>,
    truncated: bool,
    reachable_marking_count: usize,
    sound: Option<bool>,
}

/// Builds an error response envelope for a request that can't be turned into
/// a valid `WfNet` (dangling arc, non-unique source/sink, disconnected node,
/// etc.) or malformed JSON. Not part of the happy-path wire schema above --
/// deliberately named `error` so it can't be confused with a real
/// `SoundnessResponse` by a caller pattern-matching on field presence.
fn error_envelope(message: String) -> Value {
    serde_json::json!({ "error": message })
}

fn run() -> Result<Value, String> {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .map_err(|e| format!("failed to read stdin: {e}"))?;

    let request: SoundnessRequest =
        serde_json::from_str(&input).map_err(|e| format!("invalid request JSON: {e}"))?;

    let transitions = request
        .transitions
        .into_iter()
        .map(|t| (t.id, t.name));

    let mut pt = Vec::new();
    let mut tp = Vec::new();
    for arc in request.flow {
        match arc.kind {
            ArcKind::PlaceToTransition => pt.push((arc.from, arc.to)),
            ArcKind::TransitionToPlace => tp.push((arc.from, arc.to)),
        }
    }

    let net = WfNet::new(
        request.places,
        transitions,
        pt,
        tp,
        request.source,
        request.sink,
    )
    .map_err(|e| format!("not a valid WF-net: {e}"))?;

    let report = net.check_soundness();

    let sound = if report.truncated {
        None
    } else {
        Some(
            report.no_dead_transitions.unwrap_or(false)
                && report.option_to_complete.unwrap_or(false)
                && report.proper_completion.unwrap_or(false),
        )
    };

    let response = SoundnessResponse {
        no_dead_transitions: report.no_dead_transitions,
        option_to_complete: report.option_to_complete,
        proper_completion: report.proper_completion,
        is_safe: Some(report.is_safe),
        truncated: report.truncated,
        reachable_marking_count: report.reachable_markings,
        sound,
    };

    serde_json::to_value(response).map_err(|e| format!("failed to serialize response: {e}"))
}

fn main() {
    match run() {
        Ok(value) => {
            println!("{value}");
        }
        Err(message) => {
            let stderr = io::stderr();
            let mut handle = stderr.lock();
            let _ = writeln!(handle, "{}", error_envelope(message));
            std::process::exit(1);
        }
    }
}
