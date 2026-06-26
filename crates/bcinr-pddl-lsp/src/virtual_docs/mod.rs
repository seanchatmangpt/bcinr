//! Virtual document registry for bcinr-pddl:// URIs.

use serde_json::json;
use crate::lifecycle::ProjectLifecycle;
use crate::planner_client::PlanResult;
use crate::publish_gate::PublishGate;

pub const URI_LIFECYCLE: &str = "bcinr-pddl://project/lifecycle";
pub const URI_STATUS: &str = "bcinr-pddl://project/status";
pub const URI_DOMAIN: &str = "bcinr-pddl://pddl/domain";
pub const URI_PROBLEM: &str = "bcinr-pddl://pddl/problem";
pub const URI_PLAN: &str = "bcinr-pddl://pddl/plan";
pub const URI_TAPE: &str = "bcinr-pddl://pddl/tape";
pub const URI_LOG: &str = "bcinr-pddl://execution/log";
pub const URI_OCEL: &str = "bcinr-pddl://ocel/events";
pub const URI_RECEIPT: &str = "bcinr-pddl://receipt/latest";
pub const URI_PUBLISH_GATE: &str = "bcinr-pddl://publish/gate";

pub fn render_lifecycle(lc: &ProjectLifecycle) -> String {
    let stages_json: Vec<_> = lc.true_stages.iter().map(|s| s.predicate_name()).collect();
    let missing_json: Vec<_> = lc.missing.iter().map(|s| s.predicate_name()).collect();
    let evidence: Vec<_> = lc.evidence.iter().map(|e| json!({
        "stage": e.stage.predicate_name(),
        "source": e.source_path.as_ref().map(|p| p.to_string_lossy().to_string()),
        "note": e.note,
    })).collect();

    serde_json::to_string_pretty(&json!({
        "project": lc.project_name,
        "root": lc.root.to_string_lossy(),
        "true_stages": stages_json,
        "missing": missing_json,
        "evidence": evidence,
        "next_missing": lc.next_missing().map(|s| s.predicate_name()),
    })).unwrap_or_default()
}

pub fn render_status(lc: &ProjectLifecycle, gate: &PublishGate) -> String {
    serde_json::to_string_pretty(&json!({
        "project": lc.project_name,
        "lifecycle_stage_count": lc.true_stages.len(),
        "missing_count": lc.missing.len(),
        "next_step": lc.next_missing().map(|s| s.predicate_name()),
        "publish_gate": gate.status_label(),
    })).unwrap_or_default()
}

pub fn render_plan(result: &PlanResult) -> String {
    serde_json::to_string_pretty(&json!({
        "steps": result.plan_steps,
        "step_count": result.plan_steps.len(),
        "goal_reached": result.log.goal_reached,
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
    })).unwrap_or_default()
}
