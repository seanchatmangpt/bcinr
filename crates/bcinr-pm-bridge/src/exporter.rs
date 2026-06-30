//! exporter — [`WorldManufactureReceipt`] → OCEL 2.0 JSON.
//!
//! Converts plan steps from a [`WorldManufactureReceipt`] into an OCEL 2.0 JSON
//! event log conforming to the IEEE CPS 2023 specification.
//!
//! Each [`TemporalPlanStep`] becomes one OCEL event.  Object references are
//! synthesised from the step's action arguments so that the object-centric
//! structure of OCEL 2.0 is preserved.

use bcinr_pddl::WorldManufactureReceipt;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

// ─── OcelLog ─────────────────────────────────────────────────────────────────

/// A typed OCEL 2.0 event log derived from a [`WorldManufactureReceipt`].
///
/// `events` mirrors the `"ocel:events"` key of the OCEL 2.0 JSON spec.
/// `raw_json` carries the full serialisable object for export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelLog {
    /// Case/run identifier supplied to [`manufacture_with_conformance`].
    pub case_id: String,
    /// Ordered list of event descriptors (one per plan step).
    pub events: Vec<OcelEvent>,
    /// Full OCEL 2.0 JSON value ready for serialisation.
    pub raw_json: Value,
}

/// One event in the OCEL log, corresponding to one plan step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelEvent {
    /// Synthetic event id (`"<case_id>-e<seq>"`).
    pub event_id: String,
    /// Action name — maps to `"ocel:type"`.
    pub activity: String,
    /// Simulated timestamp (start_time * 1e9 nanoseconds).
    pub ts_ns: u64,
    /// Object ids synthesised from action arguments.
    pub object_ids: Vec<String>,
}

// ─── OcelExporter ────────────────────────────────────────────────────────────

/// Stateless converter: [`WorldManufactureReceipt`] → [`OcelLog`].
///
/// ## Example
///
/// ```rust
/// # use bcinr_pm_bridge::exporter::OcelExporter;
/// # use bcinr_pddl::manufacture_world;
/// let receipt = manufacture_world(
///     "(define (domain d) (:predicates (p)) (:action a :parameters () :precondition (p) :effect (not (p))))",
///     "(define (problem pr) (:domain d) (:init (p)) (:goal (not (p))))",
///     "case-x",
///     &[]
/// );
/// let log = OcelExporter::export(&receipt, "case-x");
/// assert_eq!(log.case_id, "case-x");
/// ```
pub struct OcelExporter;

impl OcelExporter {
    /// Convert a [`WorldManufactureReceipt`] into an [`OcelLog`].
    ///
    /// Steps are iterated in plan order.  Each step's `action_name` becomes the
    /// `ocel:type` (activity).  `start_time` (seconds, f64) is scaled to nanoseconds.
    /// Action arguments become object references.
    pub fn export(receipt: &WorldManufactureReceipt, case_id: &str) -> OcelLog {
        let mut events: Vec<OcelEvent> = Vec::with_capacity(receipt.plan.steps.len());
        let mut ocel_events = serde_json::Map::new();
        let mut ocel_objects = serde_json::Map::new();

        for (seq, step) in receipt.plan.steps.iter().enumerate() {
            let event_id = format!("{case_id}-e{seq}");
            let ts_ns = (step.start_time * 1_000_000_000.0) as u64;

            // Object ids: each argument becomes "<arg>" (already a ground term).
            let object_ids: Vec<String> = if step.args.is_empty() {
                // Argless actions still participate in the case object.
                vec![format!("{case_id}-case")]
            } else {
                step.args.clone()
            };

            // Build OCEL 2.0 event entry.
            let omap: Vec<Value> = object_ids
                .iter()
                .map(|id| Value::String(id.clone()))
                .collect();

            ocel_events.insert(
                event_id.clone(),
                json!({
                    "ocel:type":      step.action_name,
                    "ocel:timestamp": format!("{}.{:09}Z", ts_ns / 1_000_000_000, ts_ns % 1_000_000_000),
                    "ocel:omap":      omap,
                    "ocel:vmap":      { "duration": step.duration }
                }),
            );

            // Register each object (deduplicated by entry or_insert).
            for obj_id in &object_ids {
                ocel_objects.entry(obj_id.clone()).or_insert_with(|| {
                    json!({ "ocel:type": "pddl-object", "ocel:ovmap": {} })
                });
            }

            events.push(OcelEvent {
                event_id,
                activity: step.action_name.clone(),
                ts_ns,
                object_ids,
            });
        }

        let raw_json = json!({
            "ocel:type":            "pddl-temporal-trace",
            "ocel:attribute-names": ["activity", "duration", "ts_ns"],
            "ocel:global-log": {
                "ocel:attribute-names": ["activity", "duration"],
                "ocel:case-id":          case_id
            },
            "ocel:events":  Value::Object(ocel_events),
            "ocel:objects": Value::Object(ocel_objects)
        });

        OcelLog {
            case_id: case_id.to_owned(),
            events,
            raw_json,
        }
    }
}

// ─── Convenience wrapper ──────────────────────────────────────────────────────

/// Convenience wrapper: export OCEL log from a receipt using its problem name as
/// the case id when no explicit id is provided.
///
/// Use [`OcelExporter::export`] directly when you need a custom case id.
pub fn export_ocel_log(receipt: &WorldManufactureReceipt) -> OcelLog {
    OcelExporter::export(receipt, &receipt.problem_name)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use bcinr_pddl::manufacture_world;

    const DOMAIN: &str = r#"(define (domain d)
  (:requirements :strips)
  (:predicates (p))
  (:action a :parameters () :precondition (p) :effect (not (p))))"#;

    const PROBLEM: &str = r#"(define (problem pr)
  (:domain d)
  (:init (p))
  (:goal (not (p))))"#;

    #[test]
    fn export_produces_valid_ocel_structure() {
        let receipt = manufacture_world(DOMAIN, PROBLEM, "unit-1", &[]);
        let log = OcelExporter::export(&receipt, "unit-1");
        assert_eq!(log.case_id, "unit-1");
        // raw_json must carry the OCEL 2.0 type field.
        assert_eq!(log.raw_json["ocel:type"], "pddl-temporal-trace");
    }

    #[test]
    fn event_count_matches_plan_steps() {
        let receipt = manufacture_world(DOMAIN, PROBLEM, "unit-2", &[]);
        let log = OcelExporter::export(&receipt, "unit-2");
        assert_eq!(log.events.len(), receipt.plan.steps.len());
    }

    #[test]
    fn export_ocel_log_uses_problem_name_as_case_id() {
        let receipt = manufacture_world(DOMAIN, PROBLEM, "unit-3", &[]);
        let log = export_ocel_log(&receipt);
        assert_eq!(log.case_id, receipt.problem_name);
    }

    #[test]
    fn ts_ns_scaling() {
        let receipt = manufacture_world(DOMAIN, PROBLEM, "ts-test", &[]);
        let log = OcelExporter::export(&receipt, "ts-test");
        for (event, step) in log.events.iter().zip(receipt.plan.steps.iter()) {
            let expected = (step.start_time * 1_000_000_000.0) as u64;
            assert_eq!(event.ts_ns, expected);
        }
    }
}
