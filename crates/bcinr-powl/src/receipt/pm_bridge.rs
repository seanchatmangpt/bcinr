//! pm_bridge — OCEL 2.0 JSON serialisation bridge.
//!
//! Converts a slice of [`PowlReplayFrame`] into a `serde_json::Value` conforming to
//! the OCEL 2.0 JSON interchange format (IEEE CPS 2023 specification).
//!
//! ## Output shape
//!
//! ```json
//! {
//!   "ocel:type": "powl-causal-trace",
//!   "ocel:attribute-names": ["activity", "ts_ns"],
//!   "ocel:events": {
//!     "<event-id>": {
//!       "ocel:type": "<activity-label>",
//!       "ocel:timestamp": "<secs>.<nanos>Z",
//!       "ocel:omap": ["<object_id>", ...]
//!     }
//!   },
//!   "ocel:objects": {
//!     "<object_id>": { "ocel:type": "powl-object", "ocel:ovmap": {} }
//!   }
//! }
//! ```

use crate::receipt::replay::PowlReplayFrame;
use serde_json::{json, Value};

/// Convert a slice of [`PowlReplayFrame`] into an OCEL 2.0 JSON object.
///
/// Each frame becomes one event under `"ocel:events"`.  All distinct object ids
/// across all frames are collected into `"ocel:objects"` (duplicates deduplicated).
pub fn frames_to_ocel2_json(frames: &[PowlReplayFrame]) -> Value {
    let mut events = serde_json::Map::new();
    let mut objects = serde_json::Map::new();

    for (seq, frame) in frames.iter().enumerate() {
        let event_id = format!("e{seq}-n{}", frame.node_id);

        let omap: Vec<Value> = frame
            .object_ids
            .iter()
            .map(|id| Value::String(id.clone()))
            .collect();

        for obj_id in &frame.object_ids {
            objects
                .entry(obj_id.clone())
                .or_insert_with(|| json!({ "ocel:type": "powl-object", "ocel:ovmap": {} }));
        }

        events.insert(
            event_id,
            json!({
                "ocel:type": frame.activity,
                "ocel:timestamp": ts_ns_to_iso_stub(frame.ts_ns),
                "ocel:omap": omap
            }),
        );
    }

    json!({
        "ocel:type": "powl-causal-trace",
        "ocel:attribute-names": ["activity", "ts_ns"],
        "ocel:events": Value::Object(events),
        "ocel:objects": Value::Object(objects)
    })
}

/// Lightweight ISO 8601 stub: `"<secs>.<nanos>Z"`.
fn ts_ns_to_iso_stub(ts_ns: u64) -> String {
    format!("{}.{:09}Z", ts_ns / 1_000_000_000, ts_ns % 1_000_000_000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ts_ns_zero_produces_epoch_stub() {
        assert_eq!(ts_ns_to_iso_stub(0), "0.000000000Z");
    }

    #[test]
    fn ts_ns_one_second() {
        assert_eq!(ts_ns_to_iso_stub(1_000_000_000), "1.000000000Z");
    }

    #[test]
    fn empty_frames_yields_empty_events() {
        let v = frames_to_ocel2_json(&[]);
        assert_eq!(v["ocel:events"], json!({}));
        assert_eq!(v["ocel:objects"], json!({}));
        assert_eq!(v["ocel:type"], "powl-causal-trace");
    }
}
