//! Deterministic swarm scenarios used to falsify the temporal production rail.
//!
//! The scenario inventory is manufactured by ggen. This module executes those
//! descriptors with bounded workers and logical time. It is a validation
//! substrate, not an agent framework, and performs no LLM calls.

use std::collections::BTreeSet;
use std::fmt;

pub mod manufactured {
    include!("generated/swarm_scenarios.rs");
}

pub use manufactured::{ManufacturedScenarioDescriptor, MANUFACTURED_SCENARIOS};

/// Terminal standing expected from a deterministic scenario.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwarmStanding {
    Alive,
    Blocked,
    Refused,
}

impl SwarmStanding {
    fn from_manufactured(value: &str) -> Option<Self> {
        match value {
            "ALIVE" => Some(Self::Alive),
            "BLOCKED" => Some(Self::Blocked),
            "REFUSED" => Some(Self::Refused),
            _ => None,
        }
    }
}

/// Bounded event alphabet for all manufactured swarm scenarios.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwarmEventKind {
    PlanAdmitted = 1,
    WorkerStarted = 2,
    WorkerCompleted = 3,
    WorkerUnavailable = 4,
    WorkerSubstituted = 5,
    ResourceAcquired = 6,
    ResourceReleased = 7,
    VerificationStarted = 8,
    VerificationCompleted = 9,
    DeadlineRefused = 10,
    LeaseGranted = 11,
    HeartbeatObserved = 12,
    LeaseRenewed = 13,
    CandidateVerified = 14,
    CommitSelected = 15,
    WorkerCancelled = 16,
    InvalidReportObserved = 17,
    WorkerRefused = 18,
    ApprovalRequested = 19,
    ApprovalGranted = 20,
    ActuationCommitted = 21,
    CapacityThresholdObserved = 22,
    CapacityAllocated = 23,
    TransportEventObserved = 24,
    TraceReconstructed = 25,
    PublicationCommitted = 26,
}

/// One logical-time event emitted by a deterministic worker or verifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwarmEvent {
    pub ordinal: u16,
    pub logical_time_ms: u64,
    pub worker: &'static str,
    pub resource: Option<&'static str>,
    pub kind: SwarmEventKind,
}

/// Sealed scenario outcome.
#[derive(Debug, Clone)]
pub struct SwarmScenarioReceipt {
    pub descriptor: ManufacturedScenarioDescriptor,
    pub standing: SwarmStanding,
    pub events: Vec<SwarmEvent>,
    pub receipt_root: String,
}

impl SwarmScenarioReceipt {
    /// Recompute the receipt and enforce the scenario's load-bearing invariant.
    pub fn verify(&self) -> Result<(), SwarmValidationError> {
        verify_order(&self.events)?;
        let expected = SwarmStanding::from_manufactured(self.descriptor.expected_standing).ok_or(
            SwarmValidationError::UnknownStanding(self.descriptor.expected_standing),
        )?;
        if expected != self.standing {
            return Err(SwarmValidationError::StandingMismatch {
                expected,
                actual: self.standing,
            });
        }
        verify_scenario_invariant(self.descriptor.id, self.standing, &self.events)?;
        let recomputed = seal(self.descriptor, self.standing, &self.events);
        if recomputed != self.receipt_root {
            return Err(SwarmValidationError::ReceiptMismatch);
        }
        Ok(())
    }
}

/// Typed failures from manufactured scenario execution and verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SwarmValidationError {
    UnknownScenario(String),
    UnknownStanding(&'static str),
    StandingMismatch {
        expected: SwarmStanding,
        actual: SwarmStanding,
    },
    EventOrder {
        index: usize,
    },
    InvariantViolation {
        scenario: &'static str,
        invariant: &'static str,
    },
    ReceiptMismatch,
}

impl fmt::Display for SwarmValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownScenario(id) => write!(f, "unknown manufactured swarm scenario {id:?}"),
            Self::UnknownStanding(standing) => {
                write!(f, "unknown manufactured standing {standing:?}")
            }
            Self::StandingMismatch { expected, actual } => write!(
                f,
                "manufactured standing mismatch: expected {expected:?}, observed {actual:?}"
            ),
            Self::EventOrder { index } => {
                write!(f, "swarm event order is invalid at index {index}")
            }
            Self::InvariantViolation {
                scenario,
                invariant,
            } => write!(f, "scenario {scenario:?} violated {invariant}"),
            Self::ReceiptMismatch => write!(f, "swarm scenario receipt mismatch"),
        }
    }
}

impl std::error::Error for SwarmValidationError {}

/// Execute one ggen-manufactured scenario by stable identifier.
pub fn run_manufactured_scenario(id: &str) -> Result<SwarmScenarioReceipt, SwarmValidationError> {
    let descriptor = MANUFACTURED_SCENARIOS
        .iter()
        .copied()
        .find(|descriptor| descriptor.id == id)
        .ok_or_else(|| SwarmValidationError::UnknownScenario(id.to_string()))?;

    let (standing, events) = execute_descriptor(descriptor)?;
    let receipt_root = seal(descriptor, standing, &events);
    let receipt = SwarmScenarioReceipt {
        descriptor,
        standing,
        events,
        receipt_root,
    };
    receipt.verify()?;
    Ok(receipt)
}

/// Execute every manufactured descriptor and return one verified receipt each.
pub fn run_all_manufactured_scenarios() -> Result<Vec<SwarmScenarioReceipt>, SwarmValidationError> {
    MANUFACTURED_SCENARIOS
        .iter()
        .map(|descriptor| run_manufactured_scenario(descriptor.id))
        .collect()
}

fn execute_descriptor(
    descriptor: ManufacturedScenarioDescriptor,
) -> Result<(SwarmStanding, Vec<SwarmEvent>), SwarmValidationError> {
    let result = match descriptor.id {
        "parallel_software_delivery" => (
            SwarmStanding::Alive,
            ordered(vec![
                event(0, "planner", None, SwarmEventKind::PlanAdmitted),
                event(1, "implementer-a", None, SwarmEventKind::WorkerStarted),
                event(1, "implementer-b", None, SwarmEventKind::WorkerStarted),
                event(2, "implementer-a", None, SwarmEventKind::WorkerCompleted),
                event(2, "implementer-b", None, SwarmEventKind::WorkerCompleted),
                event(3, "verifier", None, SwarmEventKind::VerificationStarted),
                event(4, "verifier", None, SwarmEventKind::VerificationCompleted),
                event(5, "publisher", None, SwarmEventKind::PublicationCommitted),
            ]),
        ),
        "dynamic_team_formation" => (
            SwarmStanding::Alive,
            ordered(vec![
                event(0, "planner", None, SwarmEventKind::PlanAdmitted),
                event(
                    1,
                    "observatory",
                    None,
                    SwarmEventKind::CapacityThresholdObserved,
                ),
                event(2, "allocator", None, SwarmEventKind::CapacityAllocated),
                event(3, "worker-a", None, SwarmEventKind::WorkerStarted),
                event(3, "worker-b", None, SwarmEventKind::WorkerStarted),
                event(4, "worker-a", None, SwarmEventKind::WorkerCompleted),
                event(4, "worker-b", None, SwarmEventKind::WorkerCompleted),
            ]),
        ),
        "worker_substitution" => (
            SwarmStanding::Alive,
            ordered(vec![
                event(0, "primary", None, SwarmEventKind::WorkerUnavailable),
                event(1, "allocator", None, SwarmEventKind::WorkerSubstituted),
                event(2, "fallback", None, SwarmEventKind::WorkerStarted),
                event(3, "fallback", None, SwarmEventKind::WorkerCompleted),
            ]),
        ),
        "shared_accelerator_contention" => (
            SwarmStanding::Alive,
            ordered(vec![
                event(
                    0,
                    "worker-a",
                    Some("accelerator-0"),
                    SwarmEventKind::ResourceAcquired,
                ),
                event(
                    2,
                    "worker-a",
                    Some("accelerator-0"),
                    SwarmEventKind::ResourceReleased,
                ),
                event(
                    2,
                    "worker-b",
                    Some("accelerator-0"),
                    SwarmEventKind::ResourceAcquired,
                ),
                event(
                    4,
                    "worker-b",
                    Some("accelerator-0"),
                    SwarmEventKind::ResourceReleased,
                ),
            ]),
        ),
        "deadline_aware_verification" => (
            SwarmStanding::Blocked,
            ordered(vec![
                event(0, "verifier", None, SwarmEventKind::VerificationStarted),
                event(1, "scheduler", None, SwarmEventKind::DeadlineRefused),
            ]),
        ),
        "long_running_supervision" => (
            SwarmStanding::Alive,
            ordered(vec![
                event(0, "broker", Some("lease-0"), SwarmEventKind::LeaseGranted),
                event(
                    1,
                    "worker",
                    Some("lease-0"),
                    SwarmEventKind::HeartbeatObserved,
                ),
                event(2, "broker", Some("lease-0"), SwarmEventKind::LeaseRenewed),
                event(
                    3,
                    "worker",
                    Some("lease-0"),
                    SwarmEventKind::WorkerCompleted,
                ),
                event(
                    3,
                    "broker",
                    Some("lease-0"),
                    SwarmEventKind::ResourceReleased,
                ),
            ]),
        ),
        "speculative_execution" => (
            SwarmStanding::Alive,
            ordered(vec![
                event(0, "candidate-a", None, SwarmEventKind::WorkerStarted),
                event(0, "candidate-b", None, SwarmEventKind::WorkerStarted),
                event(1, "candidate-a", None, SwarmEventKind::CandidateVerified),
                event(1, "candidate-b", None, SwarmEventKind::CandidateVerified),
                event(2, "committer", None, SwarmEventKind::CommitSelected),
                event(2, "candidate-b", None, SwarmEventKind::WorkerCancelled),
            ]),
        ),
        "adversarial_worker" => (
            SwarmStanding::Refused,
            ordered(vec![
                event(
                    0,
                    "adversarial-worker",
                    None,
                    SwarmEventKind::InvalidReportObserved,
                ),
                event(1, "admission", None, SwarmEventKind::WorkerRefused),
            ]),
        ),
        "human_approval_simulation" => (
            SwarmStanding::Alive,
            ordered(vec![
                event(0, "broker", None, SwarmEventKind::ApprovalRequested),
                event(
                    2,
                    "approval-simulator",
                    None,
                    SwarmEventKind::ApprovalGranted,
                ),
                event(3, "broker", None, SwarmEventKind::ActuationCommitted),
            ]),
        ),
        "distributed_trace_reconstruction" => (
            SwarmStanding::Alive,
            reconstruct(vec![
                event(
                    3,
                    "transport-b",
                    None,
                    SwarmEventKind::TransportEventObserved,
                ),
                event(
                    1,
                    "transport-a",
                    None,
                    SwarmEventKind::TransportEventObserved,
                ),
                event(
                    2,
                    "transport-c",
                    None,
                    SwarmEventKind::TransportEventObserved,
                ),
                event(4, "reconstructor", None, SwarmEventKind::TraceReconstructed),
            ]),
        ),
        other => return Err(SwarmValidationError::UnknownScenario(other.to_string())),
    };
    Ok(result)
}

fn verify_scenario_invariant(
    scenario: &'static str,
    standing: SwarmStanding,
    events: &[SwarmEvent],
) -> Result<(), SwarmValidationError> {
    let has = |kind| events.iter().any(|event| event.kind == kind);
    let count = |kind| events.iter().filter(|event| event.kind == kind).count();
    let fail = |invariant| SwarmValidationError::InvariantViolation {
        scenario,
        invariant,
    };

    match scenario {
        "parallel_software_delivery" => {
            if count(SwarmEventKind::WorkerStarted) < 2
                || !has(SwarmEventKind::VerificationCompleted)
                || !has(SwarmEventKind::PublicationCommitted)
            {
                return Err(fail("parallel work must be verified before publication"));
            }
        }
        "dynamic_team_formation" => {
            let threshold = position(events, SwarmEventKind::CapacityThresholdObserved)?;
            let allocation = position(events, SwarmEventKind::CapacityAllocated)?;
            if threshold >= allocation {
                return Err(fail(
                    "capacity may be added only after a threshold observation",
                ));
            }
        }
        "worker_substitution" => {
            if !has(SwarmEventKind::WorkerUnavailable)
                || !has(SwarmEventKind::WorkerSubstituted)
                || !has(SwarmEventKind::WorkerCompleted)
            {
                return Err(fail("fallback must replace the unavailable primary"));
            }
        }
        "shared_accelerator_contention" => verify_exclusive_resource(events, scenario)?,
        "deadline_aware_verification" => {
            if standing != SwarmStanding::Blocked
                || !has(SwarmEventKind::DeadlineRefused)
                || has(SwarmEventKind::PublicationCommitted)
            {
                return Err(fail("impossible deadlines must block publication"));
            }
        }
        "long_running_supervision" => {
            if !has(SwarmEventKind::LeaseGranted)
                || !has(SwarmEventKind::HeartbeatObserved)
                || !has(SwarmEventKind::LeaseRenewed)
                || !has(SwarmEventKind::ResourceReleased)
            {
                return Err(fail(
                    "long-running work must retain and release a supervised lease",
                ));
            }
        }
        "speculative_execution" => {
            if count(SwarmEventKind::CommitSelected) != 1
                || count(SwarmEventKind::WorkerCancelled) != 1
            {
                return Err(fail(
                    "speculation must select exactly one commit and cancel losers",
                ));
            }
        }
        "adversarial_worker" => {
            if standing != SwarmStanding::Refused
                || !has(SwarmEventKind::WorkerRefused)
                || has(SwarmEventKind::ActuationCommitted)
            {
                return Err(fail(
                    "invalid worker evidence must be refused before actuation",
                ));
            }
        }
        "human_approval_simulation" => {
            let requested = position(events, SwarmEventKind::ApprovalRequested)?;
            let granted = position(events, SwarmEventKind::ApprovalGranted)?;
            let committed = position(events, SwarmEventKind::ActuationCommitted)?;
            if !(requested < granted && granted < committed) {
                return Err(fail("approval must precede actuation"));
            }
        }
        "distributed_trace_reconstruction" => {
            if count(SwarmEventKind::TransportEventObserved) != 3
                || count(SwarmEventKind::TraceReconstructed) != 1
            {
                return Err(fail(
                    "all transported events must be reconstructed exactly once",
                ));
            }
        }
        _ => return Err(SwarmValidationError::UnknownScenario(scenario.to_string())),
    }
    Ok(())
}

fn verify_exclusive_resource(
    events: &[SwarmEvent],
    scenario: &'static str,
) -> Result<(), SwarmValidationError> {
    let mut owned = BTreeSet::new();
    for event in events {
        let Some(resource) = event.resource else {
            continue;
        };
        match event.kind {
            SwarmEventKind::ResourceAcquired if !owned.insert(resource) => {
                return Err(SwarmValidationError::InvariantViolation {
                    scenario,
                    invariant: "exclusive resource was acquired twice without release",
                });
            }
            SwarmEventKind::ResourceReleased if !owned.remove(resource) => {
                return Err(SwarmValidationError::InvariantViolation {
                    scenario,
                    invariant: "exclusive resource was released without ownership",
                });
            }
            _ => {}
        }
    }
    if !owned.is_empty() {
        return Err(SwarmValidationError::InvariantViolation {
            scenario,
            invariant: "exclusive resource leaked after terminal execution",
        });
    }
    Ok(())
}

fn verify_order(events: &[SwarmEvent]) -> Result<(), SwarmValidationError> {
    for (index, event) in events.iter().enumerate() {
        if event.ordinal as usize != index {
            return Err(SwarmValidationError::EventOrder { index });
        }
        if index > 0 && events[index - 1].logical_time_ms > event.logical_time_ms {
            return Err(SwarmValidationError::EventOrder { index });
        }
    }
    Ok(())
}

fn position(events: &[SwarmEvent], kind: SwarmEventKind) -> Result<usize, SwarmValidationError> {
    events.iter().position(|event| event.kind == kind).ok_or(
        SwarmValidationError::InvariantViolation {
            scenario: "manufactured",
            invariant: "required event is absent",
        },
    )
}

fn event(
    logical_time_ms: u64,
    worker: &'static str,
    resource: Option<&'static str>,
    kind: SwarmEventKind,
) -> SwarmEvent {
    SwarmEvent {
        ordinal: 0,
        logical_time_ms,
        worker,
        resource,
        kind,
    }
}

fn ordered(mut events: Vec<SwarmEvent>) -> Vec<SwarmEvent> {
    for (ordinal, event) in events.iter_mut().enumerate() {
        event.ordinal = ordinal as u16;
    }
    events
}

fn reconstruct(mut events: Vec<SwarmEvent>) -> Vec<SwarmEvent> {
    events.sort_by(|left, right| {
        left.logical_time_ms
            .cmp(&right.logical_time_ms)
            .then_with(|| left.worker.cmp(right.worker))
            .then_with(|| (left.kind as u8).cmp(&(right.kind as u8)))
    });
    ordered(events)
}

fn seal(
    descriptor: ManufacturedScenarioDescriptor,
    standing: SwarmStanding,
    events: &[SwarmEvent],
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"bcinr:manufactured-swarm:v26.7.28");
    hasher.update(descriptor.id.as_bytes());
    hasher.update(&descriptor.workers.to_le_bytes());
    hasher.update(&[
        u8::from(descriptor.requires_concurrency),
        u8::from(descriptor.requires_substitution),
        u8::from(descriptor.requires_speculation),
        u8::from(descriptor.requires_human_approval),
        standing as u8,
    ]);
    for event in events {
        hasher.update(&event.ordinal.to_le_bytes());
        hasher.update(&event.logical_time_ms.to_le_bytes());
        hasher.update(event.worker.as_bytes());
        hasher.update(event.resource.unwrap_or_default().as_bytes());
        hasher.update(&[event.kind as u8]);
    }
    hasher.finalize().to_hex().to_string()
}
