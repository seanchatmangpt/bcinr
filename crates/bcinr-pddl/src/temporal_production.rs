//! Temporal PDDL → POWL geometry → admitted execution → deterministic evidence.
//!
//! This is a slow-rail composition boundary. It reuses the existing temporal
//! planner, POWL projection, Prolog8 admission, OCEL evidence, and BLAKE3
//! receipt implementations. It does not introduce an agent framework or an
//! LLM dependency.

use std::fmt;

use bcinr_mfw_ir::PlannerFailure;
use chrono::{DateTime, Utc};
use wasm4pm_compat::ocel::OCEL;
use wasm4pm_compat::pddl::{TemporalExecutionReceipt, TemporalPlan, TemporalPlanStep};

use crate::execute::execute_temporal_plan;
use crate::powl_bridge::{temporal_plan_to_powl_tape, PowlOpSpec, MAX_POWL_TAPE_STEPS};
use crate::{domain_from_pddl, problem_from_pddl, GroundTemporalProblem, Pddl8Error};

/// ggen-manufactured language-neutral runtime contract.
pub mod contract {
    include!("generated/temporal_contract.rs");
}

pub use contract::{
    TemporalRefusalCode, TemporalRuntimeProfile, CANONICAL_STATUS, LOGICAL_TIME_UNIT,
    MAXIMUM_TEMPORAL_STEPS, RECEIPT_ALGORITHM, TEMPORAL_ABI_VERSION, TEMPORAL_RECEIPT_VERSION,
    TEMPORAL_RUNTIME_PROFILE, TEMPORAL_RUNTIME_VERSION,
};

const TIME_EPSILON: f64 = 1.0e-9;
const EXECUTION_ROOT_DOMAIN: &[u8] = b"bcinr:temporal-production:v26.7.28";

/// Typed production failures for the complete temporal composition rail.
#[derive(Debug)]
pub enum TemporalProductionError {
    ParseOrGround(Pddl8Error),
    Planning(PlannerFailure),
    InvalidTime {
        step: usize,
        field: &'static str,
    },
    DurationOutOfBounds {
        step: usize,
        minimum: f64,
        maximum: f64,
        actual: f64,
    },
    UnknownAction {
        step: usize,
        action: String,
    },
    OverlappingDuplicate {
        left: usize,
        right: usize,
        action: String,
    },
    PlanBoundExceeded {
        limit: usize,
        actual: usize,
    },
    Execution(Pddl8Error),
    GoalNotReached,
    EvidenceMismatch {
        expected_events: usize,
        actual_events: usize,
    },
    ReplayMismatch,
}

impl TemporalProductionError {
    /// Stable language-neutral refusal code manufactured by ggen.
    #[must_use]
    pub const fn refusal_code(&self) -> TemporalRefusalCode {
        match self {
            Self::InvalidTime { .. } => TemporalRefusalCode::InvalidTime,
            Self::DurationOutOfBounds { .. } => TemporalRefusalCode::DurationOutOfBounds,
            Self::UnknownAction { .. } => TemporalRefusalCode::UnknownAction,
            Self::OverlappingDuplicate { .. } => TemporalRefusalCode::OverlappingDuplicate,
            Self::PlanBoundExceeded { .. } => TemporalRefusalCode::PlanBoundExceeded,
            Self::Planning(_) => TemporalRefusalCode::PlanningFailed,
            Self::ParseOrGround(_) | Self::Execution(_) => TemporalRefusalCode::ExecutionDenied,
            Self::GoalNotReached => TemporalRefusalCode::GoalNotReached,
            Self::EvidenceMismatch { .. } => TemporalRefusalCode::EvidenceMismatch,
            Self::ReplayMismatch => TemporalRefusalCode::ReplayMismatch,
        }
    }
}

impl fmt::Display for TemporalProductionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseOrGround(error) => write!(f, "temporal input was not admitted: {error}"),
            Self::Planning(error) => write!(f, "temporal planning failed: {error}"),
            Self::InvalidTime { step, field } => {
                write!(f, "temporal step {step} has invalid {field}")
            }
            Self::DurationOutOfBounds {
                step,
                minimum,
                maximum,
                actual,
            } => write!(
                f,
                "temporal step {step} duration {actual} is outside [{minimum}, {maximum}]"
            ),
            Self::UnknownAction { step, action } => {
                write!(
                    f,
                    "temporal step {step} references unknown action {action:?}"
                )
            }
            Self::OverlappingDuplicate {
                left,
                right,
                action,
            } => write!(
                f,
                "temporal steps {left} and {right} overlap the same grounded action {action:?}"
            ),
            Self::PlanBoundExceeded { limit, actual } => write!(
                f,
                "temporal plan contains {actual} steps, exceeding the admitted limit {limit}"
            ),
            Self::Execution(error) => write!(f, "temporal execution failed: {error}"),
            Self::GoalNotReached => write!(f, "temporal execution did not reach its admitted goal"),
            Self::EvidenceMismatch {
                expected_events,
                actual_events,
            } => write!(
                f,
                "temporal evidence contains {actual_events} events for {expected_events} steps"
            ),
            Self::ReplayMismatch => write!(f, "temporal execution root failed replay verification"),
        }
    }
}

impl std::error::Error for TemporalProductionError {}

impl From<Pddl8Error> for TemporalProductionError {
    fn from(value: Pddl8Error) -> Self {
        Self::ParseOrGround(value)
    }
}

/// Complete temporal execution artifact with POWL geometry and deterministic evidence.
#[derive(Debug)]
pub struct TemporalPowlExecution {
    pub plan: TemporalPlan,
    pub powl_ops: Vec<PowlOpSpec>,
    pub receipt: TemporalExecutionReceipt,
    pub ocel: OCEL,
    pub execution_root: String,
}

impl TemporalPowlExecution {
    /// Recompute the bounded execution root and validate event cardinality.
    pub fn verify(&self) -> Result<(), TemporalProductionError> {
        validate_event_alignment(&self.plan, &self.ocel)?;
        if !self.receipt.goal_reached {
            return Err(TemporalProductionError::GoalNotReached);
        }
        let recomputed = execution_root(&self.plan, &self.powl_ops, &self.receipt, &self.ocel);
        if recomputed != self.execution_root {
            return Err(TemporalProductionError::ReplayMismatch);
        }
        Ok(())
    }
}

/// Stateless deterministic temporal production runtime.
#[derive(Debug, Clone, Copy, Default)]
pub struct TemporalPowlRuntime;

impl TemporalPowlRuntime {
    /// Parse, ground, plan, project, admit, execute, seal, and normalize evidence.
    pub fn execute(
        &self,
        domain_pddl: &str,
        problem_pddl: &str,
        case_id: &str,
    ) -> Result<TemporalPowlExecution, TemporalProductionError> {
        self.execute_with_policy(domain_pddl, problem_pddl, case_id, &[])
    }

    /// Complete temporal production execution with explicit Prolog8 policy rules.
    pub fn execute_with_policy(
        &self,
        domain_pddl: &str,
        problem_pddl: &str,
        case_id: &str,
        policy_rules: &[(&str, Vec<&str>)],
    ) -> Result<TemporalPowlExecution, TemporalProductionError> {
        let domain = domain_from_pddl(domain_pddl)?;
        let problem = problem_from_pddl(problem_pddl)?;
        let grounded = GroundTemporalProblem::build(&domain, &problem)?;
        let plan = grounded
            .find_temporal_plan()
            .into_result()
            .map_err(TemporalProductionError::Planning)?;

        validate_temporal_plan_shape(&grounded, &plan)?;
        let powl_ops = temporal_plan_to_powl_tape(&plan).map_err(map_projection_error)?;
        validate_powl_geometry(&plan, &powl_ops)?;

        let (receipt, mut ocel) =
            execute_temporal_plan(&plan, &domain, &problem, case_id, policy_rules)
                .map_err(TemporalProductionError::Execution)?;

        if !receipt.goal_reached {
            return Err(TemporalProductionError::GoalNotReached);
        }

        normalize_logical_event_times(&plan, &mut ocel)?;
        validate_event_alignment(&plan, &ocel)?;
        let execution_root = execution_root(&plan, &powl_ops, &receipt, &ocel);

        let execution = TemporalPowlExecution {
            plan,
            powl_ops,
            receipt,
            ocel,
            execution_root,
        };
        execution.verify()?;
        Ok(execution)
    }
}

/// One-call convenience boundary for downstream hosts and generated bindings.
pub fn execute_temporal_pddl_to_powl(
    domain_pddl: &str,
    problem_pddl: &str,
    case_id: &str,
) -> Result<TemporalPowlExecution, TemporalProductionError> {
    TemporalPowlRuntime.execute(domain_pddl, problem_pddl, case_id)
}

/// Validate the temporal plan against grounded action identity and duration law.
pub fn validate_temporal_plan_shape(
    grounded: &GroundTemporalProblem,
    plan: &TemporalPlan,
) -> Result<(), TemporalProductionError> {
    if plan.steps.len() > MAX_POWL_TAPE_STEPS || plan.steps.len() > MAXIMUM_TEMPORAL_STEPS {
        return Err(TemporalProductionError::PlanBoundExceeded {
            limit: MAX_POWL_TAPE_STEPS.min(MAXIMUM_TEMPORAL_STEPS),
            actual: plan.steps.len(),
        });
    }
    if !plan.makespan.is_finite() || plan.makespan < 0.0 {
        return Err(TemporalProductionError::InvalidTime {
            step: plan.steps.len(),
            field: "makespan",
        });
    }

    for (index, step) in plan.steps.iter().enumerate() {
        validate_step_time(index, step)?;
        let action = grounded
            .durative_actions
            .iter()
            .find(|action| action.schema_name == step.action_name && action.args == step.args)
            .ok_or_else(|| TemporalProductionError::UnknownAction {
                step: index,
                action: canonical_action(step),
            })?;

        let below_minimum = step.duration + TIME_EPSILON < action.duration_min;
        let above_maximum =
            action.duration_max.is_finite() && step.duration > action.duration_max + TIME_EPSILON;
        if below_minimum || above_maximum {
            return Err(TemporalProductionError::DurationOutOfBounds {
                step: index,
                minimum: action.duration_min,
                maximum: action.duration_max,
                actual: step.duration,
            });
        }
    }

    for left in 0..plan.steps.len() {
        for right in (left + 1)..plan.steps.len() {
            let a = &plan.steps[left];
            let b = &plan.steps[right];
            if a.action_name == b.action_name && a.args == b.args && intervals_overlap(a, b) {
                return Err(TemporalProductionError::OverlappingDuplicate {
                    left,
                    right,
                    action: canonical_action(a),
                });
            }
        }
    }

    let computed_makespan = plan
        .steps
        .iter()
        .map(|step| step.start_time + step.duration)
        .fold(0.0_f64, f64::max);
    if (computed_makespan - plan.makespan).abs() > TIME_EPSILON {
        return Err(TemporalProductionError::InvalidTime {
            step: plan.steps.len(),
            field: "makespan consistency",
        });
    }
    Ok(())
}

fn validate_step_time(
    index: usize,
    step: &TemporalPlanStep,
) -> Result<(), TemporalProductionError> {
    if !step.start_time.is_finite() || step.start_time < 0.0 {
        return Err(TemporalProductionError::InvalidTime {
            step: index,
            field: "start time",
        });
    }
    if !step.duration.is_finite() || step.duration < 0.0 {
        return Err(TemporalProductionError::InvalidTime {
            step: index,
            field: "duration",
        });
    }
    if !(step.start_time + step.duration).is_finite() {
        return Err(TemporalProductionError::InvalidTime {
            step: index,
            field: "end time",
        });
    }
    Ok(())
}

fn validate_powl_geometry(
    plan: &TemporalPlan,
    ops: &[PowlOpSpec],
) -> Result<(), TemporalProductionError> {
    if ops.len() != plan.steps.len() {
        return Err(TemporalProductionError::EvidenceMismatch {
            expected_events: plan.steps.len(),
            actual_events: ops.len(),
        });
    }
    for (index, (step, op)) in plan.steps.iter().zip(ops).enumerate() {
        if op.start_time != Some(step.start_time)
            || op.duration != Some(step.duration)
            || op.succ_mask != (1_u64 << index)
        {
            return Err(TemporalProductionError::EvidenceMismatch {
                expected_events: plan.steps.len(),
                actual_events: ops.len(),
            });
        }
    }
    Ok(())
}

fn normalize_logical_event_times(
    plan: &TemporalPlan,
    ocel: &mut OCEL,
) -> Result<(), TemporalProductionError> {
    let ordered = ordered_steps(plan);
    if ordered.len() != ocel.events.len() {
        return Err(TemporalProductionError::EvidenceMismatch {
            expected_events: ordered.len(),
            actual_events: ocel.events.len(),
        });
    }
    for (index, (step, event)) in ordered.iter().zip(&mut ocel.events).enumerate() {
        let millis = logical_millis(index, step.start_time)?;
        let timestamp = DateTime::<Utc>::from_timestamp_millis(millis).ok_or(
            TemporalProductionError::InvalidTime {
                step: index,
                field: "logical timestamp range",
            },
        )?;
        event.time = timestamp.fixed_offset();
    }
    Ok(())
}

fn validate_event_alignment(
    plan: &TemporalPlan,
    ocel: &OCEL,
) -> Result<(), TemporalProductionError> {
    if plan.steps.len() != ocel.events.len() {
        return Err(TemporalProductionError::EvidenceMismatch {
            expected_events: plan.steps.len(),
            actual_events: ocel.events.len(),
        });
    }
    Ok(())
}

fn ordered_steps(plan: &TemporalPlan) -> Vec<TemporalPlanStep> {
    let mut ordered = plan.steps.clone();
    ordered.sort_by(|left, right| {
        left.start_time
            .total_cmp(&right.start_time)
            .then_with(|| left.action_name.cmp(&right.action_name))
            .then_with(|| left.args.cmp(&right.args))
    });
    ordered
}

fn logical_millis(step: usize, value: f64) -> Result<i64, TemporalProductionError> {
    let scaled = value * 1_000.0;
    if !scaled.is_finite() || scaled < 0.0 || scaled > i64::MAX as f64 {
        return Err(TemporalProductionError::InvalidTime {
            step,
            field: "logical millisecond conversion",
        });
    }
    Ok(scaled.round() as i64)
}

fn execution_root(
    plan: &TemporalPlan,
    ops: &[PowlOpSpec],
    receipt: &TemporalExecutionReceipt,
    ocel: &OCEL,
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(EXECUTION_ROOT_DOMAIN);
    hasher.update(TEMPORAL_RUNTIME_VERSION.as_bytes());
    hasher.update(&TEMPORAL_ABI_VERSION.to_le_bytes());
    hasher.update(&TEMPORAL_RECEIPT_VERSION.to_le_bytes());
    hasher.update(&(plan.steps.len() as u64).to_le_bytes());
    hasher.update(&plan.makespan.to_bits().to_le_bytes());
    for op in ops {
        hasher.update(op.label.as_bytes());
        hasher.update(&op.pred_mask.to_le_bytes());
        hasher.update(&op.succ_mask.to_le_bytes());
        hasher.update(&op.start_time.unwrap_or_default().to_bits().to_le_bytes());
        hasher.update(&op.duration.unwrap_or_default().to_bits().to_le_bytes());
    }
    hasher.update(receipt.plan_root.as_bytes());
    hasher.update(receipt.state_root.as_bytes());
    hasher.update(receipt.goal_root.as_bytes());
    hasher.update(&receipt.makespan.to_bits().to_le_bytes());
    hasher.update(&(receipt.step_count as u64).to_le_bytes());
    hasher.update(&[u8::from(receipt.goal_reached)]);
    hasher.update(receipt.chain_hash.as_bytes());
    for event in &ocel.events {
        hasher.update(event.id.as_bytes());
        hasher.update(event.event_type.as_bytes());
        hasher.update(&event.time.timestamp_millis().to_le_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

fn intervals_overlap(left: &TemporalPlanStep, right: &TemporalPlanStep) -> bool {
    let left_end = left.start_time + left.duration;
    let right_end = right.start_time + right.duration;
    left.start_time < right_end - TIME_EPSILON && right.start_time < left_end - TIME_EPSILON
}

fn canonical_action(step: &TemporalPlanStep) -> String {
    format!("{}({})", step.action_name, step.args.join(","))
}

fn map_projection_error(error: Pddl8Error) -> TemporalProductionError {
    match error {
        Pddl8Error::BoundExceeded { limit, got, .. } => {
            TemporalProductionError::PlanBoundExceeded { limit, actual: got }
        }
        other => TemporalProductionError::Execution(other),
    }
}
