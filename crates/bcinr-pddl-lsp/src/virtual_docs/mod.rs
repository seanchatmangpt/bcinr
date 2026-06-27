//! Virtual document registry for bcinr-pddl:// URIs.

use serde_json::json;
use crate::build_broker::BuildBrokerState;
use crate::bounds::BoundReport;
use crate::lifecycle::ProjectLifecycle;
use crate::planner_client::{PlanCandidate, PlanResult};
use crate::publish_gate::PublishGate;

// Project map URIs
pub const URI_LIFECYCLE: &str = "bcinr-pddl://project/lifecycle";
pub const URI_STATUS: &str = "bcinr-pddl://project/status";
pub const URI_EVIDENCE: &str = "bcinr-pddl://project/evidence";
pub const URI_NEXT_STEP: &str = "bcinr-pddl://project/next-step";
// Bounds
pub const URI_BOUNDS_REPORT: &str = "bcinr-pddl://bounds/report";
// PDDL8
pub const URI_DOMAIN: &str = "bcinr-pddl://pddl/domain";
pub const URI_PROBLEM: &str = "bcinr-pddl://pddl/problem";
pub const URI_PLAN: &str = "bcinr-pddl://pddl/plan";
pub const URI_TAPE: &str = "bcinr-pddl://pddl/tape";
// Execution
pub const URI_LOG: &str = "bcinr-pddl://execution/log";
pub const URI_OCEL: &str = "bcinr-pddl://ocel/events";
pub const URI_RECEIPT: &str = "bcinr-pddl://receipt/latest";
pub const URI_PUBLISH_GATE: &str = "bcinr-pddl://publish/gate";
// Build broker
pub const URI_BUILD_BROKER: &str = "bcinr-pddl://build/broker";
// Agent
pub const URI_AGENT_ASSIGNMENTS: &str = "bcinr-pddl://agent/assignments";

pub fn render_lifecycle(lc: &ProjectLifecycle) -> String {
    let stages_json: Vec<_> = lc.true_stages.iter().map(|s| s.predicate_name()).collect();
    let missing_json: Vec<_> = lc.missing.iter().map(|s| s.predicate_name()).collect();
    serde_json::to_string_pretty(&json!({
        "project": lc.project_name,
        "root": lc.root.to_string_lossy(),
        "true_stages": stages_json,
        "missing": missing_json,
        "next_missing": lc.next_missing().map(|s| s.predicate_name()),
        "stage_count": lc.true_stages.len(),
        "total_stages": crate::lifecycle::LifecycleStage::all().len(),
    })).unwrap_or_default()
}

pub fn render_status(lc: &ProjectLifecycle, gate: &PublishGate) -> String {
    serde_json::to_string_pretty(&json!({
        "project": lc.project_name,
        "lifecycle_stage_count": lc.true_stages.len(),
        "missing_count": lc.missing.len(),
        "next_step": lc.next_missing().map(|s| s.predicate_name()),
        "publish_gate": gate.status_label(),
        "admitted": gate.admitted,
        "goal_reached": gate.goal_reached,
    })).unwrap_or_default()
}

pub fn render_evidence(lc: &ProjectLifecycle) -> String {
    let evidence: Vec<_> = lc.evidence.iter().map(|e| json!({
        "stage": e.stage.predicate_name(),
        "source": e.source_path.as_ref().map(|p| p.to_string_lossy().to_string()),
        "note": e.note,
    })).collect();
    serde_json::to_string_pretty(&json!({ "project": lc.project_name, "evidence": evidence }))
        .unwrap_or_default()
}

pub fn render_next_step(lc: &ProjectLifecycle, gate: &PublishGate) -> String {
    let next = lc.next_missing().map(|s| s.predicate_name());
    serde_json::to_string_pretty(&json!({
        "project": lc.project_name,
        "next_step": next,
        "publish_gate": gate.status_label(),
        "blockers": gate.blockers,
        "instruction": next.map(|s| format!("Advance lifecycle to: {s}")),
    })).unwrap_or_default()
}

pub fn render_bounds_report(report: &BoundReport) -> String {
    let violations: Vec<_> = report.violations.iter().map(|v| json!({
        "code": v.diagnostic_code(),
        "kind": format!("{:?}", v.kind),
        "name": v.name,
        "actual": v.actual,
        "limit": v.limit,
        "message": v.message(),
    })).collect();
    serde_json::to_string_pretty(&json!({
        "violation_count": violations.len(),
        "violations": violations,
        "status": if violations.is_empty() { "OK" } else { "NEED9" },
    })).unwrap_or_default()
}

pub fn render_plan_candidate(candidate: &PlanCandidate) -> String {
    serde_json::to_string_pretty(&json!({
        "status": "CANDIDATE",
        "plan_steps": candidate.plan_steps,
        "step_count": candidate.plan_steps.len(),
        "note": "Run bcinrPddl.executeTape to admit",
    })).unwrap_or_default()
}

pub fn render_plan(result: &PlanResult) -> String {
    serde_json::to_string_pretty(&json!({
        "status": "ADMITTED",
        "steps": result.plan_steps,
        "step_count": result.plan_steps.len(),
        "goal_reached": result.log.goal_reached,
        "chain_hash": result.receipt.chain_hash,
    })).unwrap_or_default()
}

pub fn render_log(result: &PlanResult) -> String {
    let steps: Vec<_> = result.log.steps.iter().map(|s| json!({
        "index": s.op_index,
        "label": s.label,
        "admitted": s.admitted,
        "epoch_after": s.epoch_after,
        "receipt_hash": s.receipt_hash,
    })).collect();
    serde_json::to_string_pretty(&json!({
        "steps": steps,
        "goal_reached": result.log.goal_reached,
        "chain_hash": result.log.chain_hash,
    })).unwrap_or_default()
}

pub fn render_receipt(result: &PlanResult) -> String {
    serde_json::to_string_pretty(&json!({
        "plan_root": result.receipt.plan_root,
        "state_root": result.receipt.state_root,
        "goal_root": result.receipt.goal_root,
        "chain_hash": result.receipt.chain_hash,
        "goal_reached": result.receipt.goal_reached,
        "step_count": result.receipt.step_count,
    })).unwrap_or_default()
}

pub fn render_ocel(result: &PlanResult) -> String {
    serde_json::to_string_pretty(&json!({
        "event_count": result.ocel.events.len(),
        "object_count": result.ocel.objects.len(),
        "events": result.ocel.events.iter().map(|e| json!({
            "id": e.id,
            "type": e.event_type,
            "attributes": e.attributes.iter().map(|a| json!({
                "name": a.name,
                "value": a.value.to_string(),
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    })).unwrap_or_default()
}

pub fn render_publish_gate(gate: &PublishGate) -> String {
    serde_json::to_string_pretty(&json!({
        "status": gate.status_label(),
        "blockers": gate.blockers,
        "admitted": gate.is_admitted(),
        "goal_reached": gate.goal_reached,
        "receipt_hash": gate.receipt_hash,
    })).unwrap_or_default()
}

pub fn render_build_broker(state: &BuildBrokerState) -> String {
    serde_json::to_string_pretty(&json!({
        "slot_status": state.status_label(),
        "active_build": state.active_build,
        "max_slots": state.max_slots,
        "queued_count": state.queued_count,
        "denial_count": state.denial_count,
        "last_ocel_event": state.last_ocel_event,
        "can_acquire": state.can_acquire(),
    })).unwrap_or_default()
}

pub fn render_agent_assignments(lc: &ProjectLifecycle, gate: &PublishGate) -> String {
    let next_step = lc.next_missing().map(|s| s.predicate_name());
    let assignment = next_step.map(|s| format!("Advance lifecycle stage: {s}"));
    serde_json::to_string_pretty(&json!({
        "project": lc.project_name,
        "next_lawful_step": next_step,
        "assignment": assignment,
        "publish_gate": gate.status_label(),
        "admitted": gate.admitted,
        "blockers": gate.blockers,
        "instruction": "Query this document to determine the next lawful action. Do not bypass the lifecycle map.",
    })).unwrap_or_default()
}
