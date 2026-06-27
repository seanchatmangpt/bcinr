//! Publish gate — lifecycle standing determination.
//!
//! Two-pass: from_lifecycle (file evidence only) → OPEN/BLOCKED/CANDIDATE/PARTIAL
//!           from_plan_result (receipt) → ADMITTED or REFUSED
//!
//! PUBLISHED = receipt on disk with goal_reached=true (from lifecycle scan).

use crate::lifecycle::{LifecycleStage, ProjectLifecycle};
use crate::planner_client::PlanResult;

#[derive(Debug, Clone, PartialEq)]
pub enum PublishGateStatus {
    /// No lifecycle information.
    Open,
    /// Some stages present but required gates are missing.
    Blocked,
    /// Candidate plan exists but not executed yet.
    Candidate,
    /// Required lifecycle stages complete but no execution receipt.
    Partial,
    /// bcinr-pddl executed tape with goal_reached = true.
    Admitted,
    /// Receipt on disk proves goal_reached = true.
    Published,
    /// Receipt exists but goal_reached = false.
    Refused,
}

#[derive(Debug, Clone)]
pub struct PublishGate {
    pub status: PublishGateStatus,
    pub blockers: Vec<String>,
    pub admitted: bool,
    pub receipt_hash: Option<String>,
    pub goal_reached: bool,
}

impl PublishGate {
    pub fn status_label(&self) -> &'static str {
        match self.status {
            PublishGateStatus::Open => "OPEN",
            PublishGateStatus::Blocked => "BLOCKED",
            PublishGateStatus::Candidate => "CANDIDATE",
            PublishGateStatus::Partial => "PARTIAL",
            PublishGateStatus::Admitted => "ADMITTED",
            PublishGateStatus::Published => "PUBLISHED",
            PublishGateStatus::Refused => "REFUSED",
        }
    }

    pub fn is_admitted(&self) -> bool {
        self.admitted
    }
}

/// Required lifecycle stages for publish gate to be at least PARTIAL.
const REQUIRED_FOR_PARTIAL: &[LifecycleStage] = &[
    LifecycleStage::PrdAdmitted,
    LifecycleStage::ArdAdmitted,
    LifecycleStage::AdrRecorded,
    LifecycleStage::ImplementationComplete,
    LifecycleStage::TestsPassed,
    LifecycleStage::DocsProjected,
    LifecycleStage::ReleaseReady,
];

/// Compute publish gate from lifecycle state alone (before execution).
/// Result is at most PARTIAL — never ADMITTED.
pub fn from_lifecycle(lifecycle: &ProjectLifecycle) -> PublishGate {
    if lifecycle.true_stages.is_empty() {
        return PublishGate {
            status: PublishGateStatus::Open,
            blockers: vec![],
            admitted: false,
            receipt_hash: None,
            goal_reached: false,
        };
    }

    if lifecycle.has(&LifecycleStage::Published) {
        return PublishGate {
            status: PublishGateStatus::Published,
            blockers: vec![],
            admitted: true,
            receipt_hash: None,
            goal_reached: true,
        };
    }

    let blockers: Vec<String> = REQUIRED_FOR_PARTIAL
        .iter()
        .filter(|s| !lifecycle.has(s))
        .map(|s| s.predicate_name().to_string())
        .collect();

    if blockers.is_empty() {
        PublishGate {
            status: PublishGateStatus::Partial,
            blockers: vec![],
            admitted: false,
            receipt_hash: None,
            goal_reached: false,
        }
    } else {
        PublishGate {
            status: PublishGateStatus::Blocked,
            blockers,
            admitted: false,
            receipt_hash: None,
            goal_reached: false,
        }
    }
}

/// Elevate to ADMITTED if bcinr-pddl returned goal_reached=true.
/// Demote to REFUSED if execution completed but goal was not reached.
pub fn from_plan_result(lifecycle: &ProjectLifecycle, result: &PlanResult) -> PublishGate {
    if result.receipt.goal_reached {
        PublishGate {
            status: PublishGateStatus::Admitted,
            blockers: vec![],
            admitted: true,
            receipt_hash: Some(result.receipt.chain_hash.clone()),
            goal_reached: true,
        }
    } else {
        let base = from_lifecycle(lifecycle);
        let mut blockers = base.blockers.clone();
        blockers.push("goal_not_reached".into());
        PublishGate {
            status: PublishGateStatus::Refused,
            blockers,
            admitted: false,
            receipt_hash: Some(result.receipt.chain_hash.clone()),
            goal_reached: false,
        }
    }
}
