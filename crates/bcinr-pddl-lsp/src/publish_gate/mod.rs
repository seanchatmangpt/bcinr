//! Publish gate — final lifecycle check before `published(project)` is admitted.

use crate::lifecycle::{LifecycleStage, ProjectLifecycle};
use crate::planner_client::PlanResult;

#[derive(Debug, Clone, PartialEq)]
pub enum PublishGateStatus {
    Open,
    Partial,
    Blocked,
    Admitted,
    Refused,
    Published,
}

#[derive(Debug, Clone)]
pub struct PublishGate {
    pub status: PublishGateStatus,
    pub blockers: Vec<String>,
}

impl PublishGate {
    pub fn status_label(&self) -> &'static str {
        match self.status {
            PublishGateStatus::Open => "OPEN",
            PublishGateStatus::Partial => "PARTIAL",
            PublishGateStatus::Blocked => "BLOCKED",
            PublishGateStatus::Admitted => "ADMITTED",
            PublishGateStatus::Refused => "REFUSED",
            PublishGateStatus::Published => "PUBLISHED",
        }
    }

    pub fn is_admitted(&self) -> bool {
        matches!(self.status, PublishGateStatus::Admitted | PublishGateStatus::Published)
    }
}

/// Compute publish gate from lifecycle state alone (before planning).
pub fn from_lifecycle(lifecycle: &ProjectLifecycle) -> PublishGate {
    let required = [
        LifecycleStage::PrdAdmitted,
        LifecycleStage::ArdAdmitted,
        LifecycleStage::ImplementationComplete,
        LifecycleStage::TestsPassed,
        LifecycleStage::DocsProjected,
        LifecycleStage::ReleaseReady,
    ];

    let blockers: Vec<String> = required
        .iter()
        .filter(|s| !lifecycle.has(s))
        .map(|s| s.predicate_name().to_string())
        .collect();

    if lifecycle.has(&LifecycleStage::Published) {
        PublishGate { status: PublishGateStatus::Published, blockers: vec![] }
    } else if blockers.is_empty() {
        PublishGate { status: PublishGateStatus::Partial, blockers: vec![] }
    } else {
        PublishGate { status: PublishGateStatus::Blocked, blockers }
    }
}

/// Elevate gate to ADMITTED if bcinr-pddl execution succeeded with receipt.
pub fn from_plan_result(lifecycle: &ProjectLifecycle, result: &PlanResult) -> PublishGate {
    let base = from_lifecycle(lifecycle);
    if result.receipt.goal_reached {
        PublishGate { status: PublishGateStatus::Admitted, blockers: vec![] }
    } else {
        let mut blockers = base.blockers.clone();
        blockers.push("goal_not_reached".into());
        PublishGate { status: PublishGateStatus::Blocked, blockers }
    }
}
