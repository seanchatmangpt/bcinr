//! Object-centric event log generation: turn a chain run into OCEL events.
//!
//! A chain run produces eight kernel outputs (`step1..step8`). This module maps each step
//! onto an [`OcelEvent`] whose `activity` is the pattern id, `event_code` is the pattern's
//! [`EventKind`] code, `timestamp` is the step index, and whose object links come from the
//! pattern's declared [`ObjectKind`]s — so each event is traceable to a real object and the
//! log is not an "OCEL-laundering" stream of object-less events.
//!
//! [`EventKind`]: crate::ir::EventKind
//! [`ObjectKind`]: crate::ir::ObjectKind

use super::model::ChainModel;
use super::Trace;
use crate::class::status;
use crate::evidence::ocel::OcelEvent;
use crate::ir::PatternSpec;
use crate::patterns::PATTERN_REGISTRY;

/// Look up a pattern spec by its id, scanning [`PATTERN_REGISTRY`].
///
/// Returns `None` if no pattern carries `id` (e.g. an out-of-range or sentinel id).
#[must_use]
pub fn spec_by_id(id: u16) -> Option<&'static PatternSpec> {
    PATTERN_REGISTRY.iter().find(|s| s.id.raw() == id)
}

/// Build the eight OCEL events for one chain run.
///
/// `kernel_outputs[i]` is step `i + 1`'s `u64` result; it becomes the object id linked to
/// the event, so the event points at the value it produced. Each event's object types are
/// the pattern's declared object kinds (bounded by [`crate::evidence::ocel::ObjectRefs::CAP`]).
#[must_use = "returns the 8-event OCEL log for the chain run"]
pub fn events_for_chain(model: &ChainModel, kernel_outputs: &[u64; 8]) -> [OcelEvent; 8] {
    let mut out = [OcelEvent::new(0, 0, 0, status::UNKNOWN); 8];
    for (i, slot) in out.iter_mut().enumerate() {
        let pid = model.activities[i];
        let mut ev = OcelEvent::new(0, pid, i as u64, status::ADMITTED);
        if let Some(spec) = spec_by_id(pid) {
            ev.event_code = spec.event.code;
            for obj in spec.objects {
                ev.objects.push(obj.code, kernel_outputs[i]);
            }
        }
        *slot = ev;
    }
    out
}

/// Extract the observed [`Trace`] (activity sequence) from an OCEL event slice.
#[must_use = "returns the trace; feed it to the conformance checker"]
pub fn trace_for_chain(events: &[OcelEvent]) -> Trace {
    let mut trace = Trace::new();
    for ev in events {
        trace.push(ev.activity);
    }
    trace
}

#[cfg(test)]
mod tests {
    use super::super::conformance::{check_trace, to_verdict};
    use super::super::model::CHAIN_MODELS;
    use super::*;
    use crate::compat::Verdict;

    #[test]
    fn events_carry_pattern_id_event_code_timestamp_and_objects() {
        let m = &CHAIN_MODELS[2]; // combat_hit
        let outputs = [4u64, 1, 86, 2, 1, 0, 0xdead_beef, 4];
        let events = events_for_chain(m, &outputs);
        assert_eq!(events.len(), 8);
        for (i, ev) in events.iter().enumerate() {
            let pid = m.activities[i];
            assert_eq!(ev.activity, pid);
            assert_eq!(ev.timestamp, i as u64);
            let spec = spec_by_id(pid).expect("every chain activity is a real pattern");
            assert_eq!(ev.event_code, spec.event.code);
            assert_eq!(ev.objects.len(), spec.objects.len());
            if !spec.objects.is_empty() {
                assert!(
                    !ev.objects.is_empty(),
                    "object-linked event must not be empty"
                );
            }
        }
    }

    #[test]
    fn trace_round_trips_to_model_and_self_conforms() {
        for m in CHAIN_MODELS {
            let outputs = [0u64; 8];
            let events = events_for_chain(m, &outputs);
            let trace = trace_for_chain(&events);
            assert_eq!(trace.as_slice(), m.activities.as_slice());
            let r = check_trace(m, &trace);
            assert_eq!(
                r.fitness_bp, 10_000,
                "chain {} OCEL trace must conform",
                m.name
            );
            assert!(matches!(to_verdict(&r), Verdict::Admitted));
        }
    }

    #[test]
    fn unknown_pattern_id_has_no_spec() {
        assert!(spec_by_id(u16::MAX).is_none());
        assert!(spec_by_id(9999).is_none());
    }
}
