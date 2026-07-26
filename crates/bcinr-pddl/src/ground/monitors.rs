//! Deterministic trajectory constraint monitors.
//!
//! # Architecture
//!
//! Each monitor is a state machine consuming (M_t, S_t, a_t, S_{t+1}) and producing M_{t+1}.
//! Monitors implement the `ConstraintMonitor` trait and are compiled once per constraint kind,
//! not special-cased.
//!
//! # States
//!
//! - **Satisfied**: Constraint is fully satisfied and cannot be violated.
//! - **Pending**: Constraint is not yet satisfied but could still become satisfied.
//! - **Violated**: Constraint was violated at this step.
//! - **IrrecoverablyViolated**: Constraint can never be satisfied from this point forward.
//!
//! # Hard vs Soft Constraints
//!
//! - **Hard constraints** (from `:constraints` section): Plans violating any hard constraint are refused.
//! - **Soft preferences** (from `:preferences` section): Violations contribute to the plan cost/metric.

use std::collections::BTreeSet;

use wasm4pm_compat::pddl::{Pddl8GroundAtom, PddlCondition, TrajectoryConstraint};

use crate::ground::{
    eval_condition, QuantifierDomain, GroundDerivedPredicate,
};
use std::collections::HashMap;

/// State of a constraint monitor after processing one step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitorState {
    /// Constraint is fully satisfied and cannot be violated.
    Satisfied,
    /// Constraint is not yet satisfied but could still become satisfied.
    Pending,
    /// Constraint was violated at this step.
    Violated,
    /// Constraint can never be satisfied from this point forward.
    IrrecoverablyViolated,
}

impl MonitorState {
    /// Check if the constraint is violated (either now or irrecoverably).
    pub fn is_violated(&self) -> bool {
        matches!(self, MonitorState::Violated | MonitorState::IrrecoverablyViolated)
    }

    /// Check if the constraint is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(self, MonitorState::Satisfied | MonitorState::IrrecoverablyViolated)
    }
}

/// A monitor processes (M_t, S_t, a_t, S_{t+1}) and produces M_{t+1}.
///
/// `M_t` is the monitor's internal state (what happened before this step).
/// `S_t` is the state before the action.
/// `a_t` is the action executed (None if a timed initial literal event).
/// `S_{t+1}` is the state after the action.
pub trait ConstraintMonitor: Send + Sync {
    /// Process a single step: M_t + (S_t, a_t, S_{t+1}) -> (M_{t+1}, outcome).
    ///
    /// Returns the new monitor state and the outcome.
    fn step(
        &self,
        monitor_state: MonitorState,
        prev_state: &BTreeSet<Pddl8GroundAtom>,
        action_taken: Option<&str>, // None = timed initial literal
        next_state: &BTreeSet<Pddl8GroundAtom>,
        fn_values: &HashMap<String, f64>,
        quant_domain: &QuantifierDomain,
        derived_predicates: &[GroundDerivedPredicate],
    ) -> MonitorState;
}

/// `(always (c))`: Condition `c` must hold in every state.
pub struct AlwaysMonitor {
    condition: PddlCondition,
}

impl AlwaysMonitor {
    pub fn new(condition: PddlCondition) -> Self {
        Self { condition }
    }
}

impl ConstraintMonitor for AlwaysMonitor {
    fn step(
        &self,
        monitor_state: MonitorState,
        _prev_state: &BTreeSet<Pddl8GroundAtom>,
        _action_taken: Option<&str>,
        next_state: &BTreeSet<Pddl8GroundAtom>,
        fn_values: &HashMap<String, f64>,
        quant_domain: &QuantifierDomain,
        _derived_predicates: &[GroundDerivedPredicate],
    ) -> MonitorState {
        // Once satisfied, always remains satisfied (but check each step)
        if monitor_state == MonitorState::Satisfied {
            return MonitorState::Satisfied;
        }

        if monitor_state.is_terminal() {
            return monitor_state;
        }

        // Check if condition holds in the new state
        if eval_condition(&self.condition, next_state, fn_values, quant_domain) {
            MonitorState::Pending // Keep checking
        } else {
            MonitorState::Violated
        }
    }
}

/// `(sometime (c))`: Condition `c` must hold in at least one state.
pub struct SometimeMonitor {
    condition: PddlCondition,
}

impl SometimeMonitor {
    pub fn new(condition: PddlCondition) -> Self {
        Self { condition }
    }
}

impl ConstraintMonitor for SometimeMonitor {
    fn step(
        &self,
        monitor_state: MonitorState,
        _prev_state: &BTreeSet<Pddl8GroundAtom>,
        _action_taken: Option<&str>,
        next_state: &BTreeSet<Pddl8GroundAtom>,
        fn_values: &HashMap<String, f64>,
        quant_domain: &QuantifierDomain,
        _derived_predicates: &[GroundDerivedPredicate],
    ) -> MonitorState {
        // Once satisfied, always satisfied
        if monitor_state == MonitorState::Satisfied {
            return MonitorState::Satisfied;
        }

        // Check if condition holds in the new state
        if eval_condition(&self.condition, next_state, fn_values, quant_domain) {
            MonitorState::Satisfied
        } else {
            MonitorState::Pending
        }
    }
}

/// `(within n (c))`: Condition `c` must hold within n time steps.
pub struct WithinMonitor {
    steps_remaining: i64,
    condition: PddlCondition,
}

impl WithinMonitor {
    pub fn new(time_limit: f64, condition: PddlCondition) -> Self {
        Self {
            steps_remaining: time_limit.ceil() as i64,
            condition,
        }
    }
}

impl ConstraintMonitor for WithinMonitor {
    fn step(
        &self,
        monitor_state: MonitorState,
        _prev_state: &BTreeSet<Pddl8GroundAtom>,
        _action_taken: Option<&str>,
        next_state: &BTreeSet<Pddl8GroundAtom>,
        fn_values: &HashMap<String, f64>,
        quant_domain: &QuantifierDomain,
        _derived_predicates: &[GroundDerivedPredicate],
    ) -> MonitorState {
        // Once satisfied, always satisfied
        if monitor_state == MonitorState::Satisfied {
            return MonitorState::Satisfied;
        }

        let new_remaining = self.steps_remaining - 1;

        // Check if condition holds
        if eval_condition(&self.condition, next_state, fn_values, quant_domain) {
            return MonitorState::Satisfied;
        }

        // If time ran out, irrecoverably violated
        if new_remaining <= 0 {
            return MonitorState::IrrecoverablyViolated;
        }

        // Still waiting (but this won't work for state machines that need to track steps)
        // For a real implementation, step() would need to know the current time
        MonitorState::Pending
    }
}

/// `(at-most-once (c))`: Condition `c` must hold at most once across all states.
///
/// MonitorState encodes:
/// - Pending: Haven't seen the condition yet
/// - Satisfied: Saw the condition once and it's now false (or still false)
/// - Violated: Saw the condition multiple times (it held, then was false, then held again)
pub struct AtMostOnceMonitor {
    condition: PddlCondition,
}

impl AtMostOnceMonitor {
    pub fn new(condition: PddlCondition) -> Self {
        Self { condition }
    }
}

impl ConstraintMonitor for AtMostOnceMonitor {
    fn step(
        &self,
        monitor_state: MonitorState,
        prev_state: &BTreeSet<Pddl8GroundAtom>,
        _action_taken: Option<&str>,
        next_state: &BTreeSet<Pddl8GroundAtom>,
        fn_values: &HashMap<String, f64>,
        quant_domain: &QuantifierDomain,
        _derived_predicates: &[GroundDerivedPredicate],
    ) -> MonitorState {
        if monitor_state.is_terminal() {
            return monitor_state;
        }

        let _prev_holds = eval_condition(&self.condition, prev_state, fn_values, quant_domain);
        let next_holds = eval_condition(&self.condition, next_state, fn_values, quant_domain);

        match monitor_state {
            MonitorState::Pending => {
                if next_holds {
                    // First time seeing the condition -> satisfied (for now)
                    MonitorState::Satisfied
                } else {
                    // Still waiting for the first occurrence
                    MonitorState::Pending
                }
            }
            MonitorState::Satisfied => {
                if next_holds {
                    // Condition was satisfied once and now holds again -> violated
                    MonitorState::Violated
                } else {
                    // Still satisfied (condition doesn't hold yet or remains false)
                    MonitorState::Satisfied
                }
            }
            _ => monitor_state,
        }
    }
}

/// `(sometime-before (c1) (c2))`: c1 must hold before c2.
///
/// MonitorState encodes:
/// - Pending: Haven't seen either c1 or c2 yet
/// - Satisfied: Saw c1 before c2 (constraint satisfied)
/// - Violated: Saw c2 before c1 (constraint violated)
pub struct SometimeBeforeMonitor {
    condition_before: PddlCondition,
    condition_after: PddlCondition,
}

impl SometimeBeforeMonitor {
    pub fn new(condition_before: PddlCondition, condition_after: PddlCondition) -> Self {
        Self {
            condition_before,
            condition_after,
        }
    }
}

impl ConstraintMonitor for SometimeBeforeMonitor {
    fn step(
        &self,
        monitor_state: MonitorState,
        _prev_state: &BTreeSet<Pddl8GroundAtom>,
        _action_taken: Option<&str>,
        next_state: &BTreeSet<Pddl8GroundAtom>,
        fn_values: &HashMap<String, f64>,
        quant_domain: &QuantifierDomain,
        _derived_predicates: &[GroundDerivedPredicate],
    ) -> MonitorState {
        if monitor_state.is_terminal() {
            return monitor_state;
        }

        let after_holds = eval_condition(&self.condition_after, next_state, fn_values, quant_domain);
        let before_holds =
            eval_condition(&self.condition_before, next_state, fn_values, quant_domain);

        if after_holds && monitor_state == MonitorState::Pending {
            // c2 holds but we haven't seen c1 yet -> violated
            MonitorState::Violated
        } else if before_holds {
            // c1 holds -> satisfied (we've seen c1, so even if c2 holds, constraint is satisfied)
            MonitorState::Satisfied
        } else {
            MonitorState::Pending
        }
    }
}

/// `(sometime-after (c1) (c2))`: c2 must hold after c1.
///
/// MonitorState encodes:
/// - Pending: Haven't seen c1 yet
/// - Satisfied: Saw c1 then c2 (constraint satisfied)
/// - IrrecoverablyViolated: Saw c1 but plan ends without seeing c2 (can only check at finalize)
pub struct SometimeAfterMonitor {
    condition_before: PddlCondition,
    condition_after: PddlCondition,
}

impl SometimeAfterMonitor {
    pub fn new(condition_before: PddlCondition, condition_after: PddlCondition) -> Self {
        Self {
            condition_before,
            condition_after,
        }
    }
}

impl ConstraintMonitor for SometimeAfterMonitor {
    fn step(
        &self,
        monitor_state: MonitorState,
        _prev_state: &BTreeSet<Pddl8GroundAtom>,
        _action_taken: Option<&str>,
        next_state: &BTreeSet<Pddl8GroundAtom>,
        fn_values: &HashMap<String, f64>,
        quant_domain: &QuantifierDomain,
        _derived_predicates: &[GroundDerivedPredicate],
    ) -> MonitorState {
        if monitor_state.is_terminal() {
            return monitor_state;
        }

        let before_holds = eval_condition(&self.condition_before, next_state, fn_values, quant_domain);
        let after_holds = eval_condition(&self.condition_after, next_state, fn_values, quant_domain);

        match monitor_state {
            MonitorState::Pending => {
                if before_holds {
                    // We've seen c1, now waiting for c2
                    // Return Pending to indicate we're tracking
                    if after_holds {
                        // Immediately saw c2 after c1 -> satisfied
                        MonitorState::Satisfied
                    } else {
                        // c1 holds but c2 doesn't yet
                        MonitorState::Pending
                    }
                } else {
                    // Haven't seen c1 yet
                    MonitorState::Pending
                }
            }
            _ => monitor_state,
        }
    }
}

/// `(always-within n (c1) (c2))`: c2 must hold within n time steps of c1.
pub struct AlwaysWithinMonitor {
    time_limit: f64,
    condition_trigger: PddlCondition,
    condition_target: PddlCondition,
    triggered_at: Option<f64>,
}

impl AlwaysWithinMonitor {
    pub fn new(
        time_limit: f64,
        condition_trigger: PddlCondition,
        condition_target: PddlCondition,
    ) -> Self {
        Self {
            time_limit,
            condition_trigger,
            condition_target,
            triggered_at: None,
        }
    }
}

impl ConstraintMonitor for AlwaysWithinMonitor {
    fn step(
        &self,
        monitor_state: MonitorState,
        _prev_state: &BTreeSet<Pddl8GroundAtom>,
        _action_taken: Option<&str>,
        next_state: &BTreeSet<Pddl8GroundAtom>,
        fn_values: &HashMap<String, f64>,
        quant_domain: &QuantifierDomain,
        _derived_predicates: &[GroundDerivedPredicate],
    ) -> MonitorState {
        if monitor_state.is_terminal() {
            return monitor_state;
        }

        let trigger_holds =
            eval_condition(&self.condition_trigger, next_state, fn_values, quant_domain);
        let target_holds =
            eval_condition(&self.condition_target, next_state, fn_values, quant_domain);

        // This implementation would need access to current time to properly track the window
        // For now, check if target holds when trigger holds
        if trigger_holds && !target_holds {
            MonitorState::Violated
        } else if trigger_holds && target_holds {
            MonitorState::Satisfied
        } else {
            MonitorState::Pending
        }
    }
}

/// Factory for creating monitors from TrajectoryConstraint.
pub struct MonitorFactory;

impl MonitorFactory {
    /// Create a monitor for the given trajectory constraint.
    ///
    /// Returns None if the constraint type is not yet supported.
    pub fn create_monitor(constraint: &TrajectoryConstraint) -> Option<Box<dyn ConstraintMonitor>> {
        match constraint {
            TrajectoryConstraint::Always(cond) => {
                Some(Box::new(AlwaysMonitor::new((**cond).clone())))
            }
            TrajectoryConstraint::Sometime(cond) => {
                Some(Box::new(SometimeMonitor::new((**cond).clone())))
            }
            TrajectoryConstraint::Within(time_limit, cond) => {
                Some(Box::new(WithinMonitor::new(*time_limit, (**cond).clone())))
            }
            TrajectoryConstraint::AtMostOnce(cond) => {
                Some(Box::new(AtMostOnceMonitor::new((**cond).clone())))
            }
            TrajectoryConstraint::SometimeBefore(c1, c2) => {
                Some(Box::new(SometimeBeforeMonitor::new(
                    (**c1).clone(),
                    (**c2).clone(),
                )))
            }
            TrajectoryConstraint::SometimeAfter(c1, c2) => {
                Some(Box::new(SometimeAfterMonitor::new(
                    (**c1).clone(),
                    (**c2).clone(),
                )))
            }
            TrajectoryConstraint::AlwaysWithin(time_limit, c1, c2) => {
                Some(Box::new(AlwaysWithinMonitor::new(
                    *time_limit,
                    (**c1).clone(),
                    (**c2).clone(),
                )))
            }
            TrajectoryConstraint::And(_parts) => {
                // Conjunction should be handled at the constraint collection level
                None
            }
            TrajectoryConstraint::HoldDuring(_, _, _) | TrajectoryConstraint::HoldAfter(_, _) => {
                // These are extensions beyond the core 7 types
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_state_is_violated() {
        assert!(MonitorState::Violated.is_violated());
        assert!(MonitorState::IrrecoverablyViolated.is_violated());
        assert!(!MonitorState::Pending.is_violated());
        assert!(!MonitorState::Satisfied.is_violated());
    }

    #[test]
    fn test_monitor_state_is_terminal() {
        assert!(MonitorState::Satisfied.is_terminal());
        assert!(MonitorState::IrrecoverablyViolated.is_terminal());
        assert!(!MonitorState::Pending.is_terminal());
        assert!(!MonitorState::Violated.is_terminal());
    }
}
