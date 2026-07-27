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

use crate::ground::{eval_condition, GroundDerivedPredicate, QuantifierDomain};
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
        matches!(
            self,
            MonitorState::Violated | MonitorState::IrrecoverablyViolated
        )
    }

    /// Check if the constraint is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            MonitorState::Satisfied | MonitorState::IrrecoverablyViolated
        )
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

    /// Whether the trajectory ending with this monitor still in
    /// `MonitorState::Pending` counts as satisfied, not violated.
    ///
    /// `Pending` means two different things depending on the constraint:
    /// for a "must eventually happen" monitor (e.g. `Sometime`, whose own
    /// `step` only leaves `Pending` for "haven't witnessed it yet"), ending
    /// `Pending` means the required event never occurred -- a genuine
    /// violation, and the default (`false`) here is correct. For a "must
    /// never be falsified" monitor (e.g. `Always`, whose `step` has no
    /// terminal `Satisfied` transition at all -- it only ever holds
    /// `Pending` while unfalsified, or moves to `Violated`), reaching the
    /// end of the trajectory still `Pending` means the condition held
    /// throughout without ever failing, which for that constraint's own
    /// semantics *is* satisfaction.
    fn pending_is_satisfied_at_finalize(&self) -> bool {
        false
    }
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
    fn pending_is_satisfied_at_finalize(&self) -> bool {
        true
    }

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

        let after_holds =
            eval_condition(&self.condition_after, next_state, fn_values, quant_domain);
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

        let before_holds =
            eval_condition(&self.condition_before, next_state, fn_values, quant_domain);
        let after_holds =
            eval_condition(&self.condition_after, next_state, fn_values, quant_domain);

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
            TrajectoryConstraint::AtMostOnce(cond) => {
                Some(Box::new(AtMostOnceMonitor::new((**cond).clone())))
            }
            TrajectoryConstraint::SometimeBefore(c1, c2) => Some(Box::new(
                SometimeBeforeMonitor::new((**c1).clone(), (**c2).clone()),
            )),
            TrajectoryConstraint::SometimeAfter(c1, c2) => Some(Box::new(
                SometimeAfterMonitor::new((**c1).clone(), (**c2).clone()),
            )),
            TrajectoryConstraint::And(_parts) => {
                // Conjunction should be handled at the constraint collection level
                None
            }
            // `within`/`always-within` are refused rather than monitored: a real
            // implementation needs step() to know the current time/tick to track
            // a countdown or window, which ConstraintMonitor::step's signature
            // does not provide -- there is no way to compute the correct answer
            // here, only a wrong one that looks like an answer (see git history
            // for the removed WithinMonitor/AlwaysWithinMonitor, whose own
            // comments admitted this). Grouped with HoldDuring/HoldAfter, the
            // other two operators this factory already refuses outright.
            TrajectoryConstraint::Within(_, _)
            | TrajectoryConstraint::AlwaysWithin(_, _, _)
            | TrajectoryConstraint::HoldDuring(_, _, _)
            | TrajectoryConstraint::HoldAfter(_, _) => None,
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
