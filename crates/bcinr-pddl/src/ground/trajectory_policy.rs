//! Temporal policy closure: trajectory constraint + preference monitoring.
//!
//! This module integrates trajectory constraints and preferences into the planning process.
//! It distinguishes between hard constraints (must be satisfied) and soft preferences
//! (violations add to cost).

use std::collections::{BTreeSet, HashMap};

use wasm4pm_compat::pddl::{Pddl8GroundAtom, PddlPreference, TrajectoryConstraint};

use crate::ground::{
    monitors::{ConstraintMonitor, MonitorFactory, MonitorState},
    GroundDerivedPredicate, QuantifierDomain,
};

/// A typed refusal reason for constraint violations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintViolation {
    /// Hard constraint was violated (plans violating hard constraints are refused).
    HardConstraint,
    /// Soft preference was violated (violations add to plan cost).
    SoftPreference,
}

/// Trajectory policy: monitors all constraints and preferences in a problem.
#[allow(dead_code)]
pub struct TrajectoryPolicy {
    /// Hard constraints: violations refuse the plan.
    hard_monitors: Vec<(String, Box<dyn ConstraintMonitor>)>,
    /// Soft preferences: violations add to cost.
    soft_monitors: Vec<(String, Box<dyn ConstraintMonitor>)>,
    /// Current states of all monitors.
    hard_states: Vec<MonitorState>,
    soft_states: Vec<MonitorState>,
    /// Violation count for soft preferences (used for cost calculation).
    soft_violations: usize,
}

impl TrajectoryPolicy {
    /// Create a new trajectory policy from a list of preferences and hard constraints.
    ///
    /// # Arguments
    ///
    /// * `preferences` - Soft preferences (violations contribute to cost)
    /// * `hard_constraints` - Hard constraints (violations refuse the plan)
    pub fn new(preferences: &[PddlPreference], hard_constraints: &[TrajectoryConstraint]) -> Self {
        let mut hard_monitors = Vec::new();
        let mut soft_monitors = Vec::new();

        // Initialize hard constraint monitors
        for (idx, constraint) in hard_constraints.iter().enumerate() {
            if let Some(monitor) = MonitorFactory::create_monitor(constraint) {
                hard_monitors.push((format!("hard_{}", idx), monitor));
            }
        }

        // Initialize soft preference monitors
        for (idx, pref) in preferences.iter().enumerate() {
            if let Some(monitor) = MonitorFactory::create_monitor(&pref.constraint) {
                soft_monitors.push((format!("pref_{}", idx), monitor));
            }
        }

        let hard_states = vec![MonitorState::Pending; hard_monitors.len()];
        let soft_states = vec![MonitorState::Pending; soft_monitors.len()];

        Self {
            hard_monitors,
            soft_monitors,
            hard_states,
            soft_states,
            soft_violations: 0,
        }
    }

    /// Process a single step: update all monitor states.
    ///
    /// Returns `Ok(violation_count)` if no hard constraints were violated,
    /// or `Err(ConstraintViolation::HardConstraint)` if a hard constraint was violated.
    pub fn step(
        &mut self,
        prev_state: &BTreeSet<Pddl8GroundAtom>,
        action_taken: Option<&str>,
        next_state: &BTreeSet<Pddl8GroundAtom>,
        fn_values: &HashMap<String, f64>,
        quant_domain: &QuantifierDomain,
        derived_predicates: &[GroundDerivedPredicate],
    ) -> Result<usize, ConstraintViolation> {
        // Update hard constraint monitors
        for (idx, (_, monitor)) in self.hard_monitors.iter().enumerate() {
            let new_state = monitor.step(
                self.hard_states[idx],
                prev_state,
                action_taken,
                next_state,
                fn_values,
                quant_domain,
                derived_predicates,
            );
            self.hard_states[idx] = new_state;

            // Hard constraint violated -> refuse plan
            if new_state == MonitorState::Violated
                || new_state == MonitorState::IrrecoverablyViolated
            {
                return Err(ConstraintViolation::HardConstraint);
            }
        }

        // Update soft preference monitors
        for (idx, (_, monitor)) in self.soft_monitors.iter().enumerate() {
            let new_state = monitor.step(
                self.soft_states[idx],
                prev_state,
                action_taken,
                next_state,
                fn_values,
                quant_domain,
                derived_predicates,
            );
            self.soft_states[idx] = new_state;

            // Count violations
            if new_state == MonitorState::Violated {
                self.soft_violations += 1;
            }
        }

        Ok(self.soft_violations)
    }

    /// Check all hard constraints at the end of the plan.
    ///
    /// Some constraints (like `sometime`) can only be fully checked at the end.
    /// `Pending` is not uniformly a violation here: each monitor's own
    /// `pending_is_satisfied_at_finalize` says whether ending `Pending` means
    /// "never witnessed" (a violation, e.g. `sometime`) or "held throughout
    /// without ever failing" (satisfied, e.g. `always`) -- see that method's
    /// doc comment.
    pub fn finalize(&self) -> Result<(), ConstraintViolation> {
        for (state, (_, monitor)) in self.hard_states.iter().zip(&self.hard_monitors) {
            match state {
                MonitorState::Pending => {
                    if !monitor.pending_is_satisfied_at_finalize() {
                        return Err(ConstraintViolation::HardConstraint);
                    }
                }
                MonitorState::Violated | MonitorState::IrrecoverablyViolated => {
                    return Err(ConstraintViolation::HardConstraint);
                }
                MonitorState::Satisfied => {} // OK
            }
        }

        Ok(())
    }

    /// Get the final violation count for soft preferences.
    pub fn soft_violation_count(&self) -> usize {
        self.soft_violations
    }

    /// Get the number of hard constraints that are not yet satisfied.
    pub fn pending_hard_constraints(&self) -> usize {
        self.hard_states
            .iter()
            .filter(|s| **s == MonitorState::Pending)
            .count()
    }

    /// Get the number of satisfied hard constraints.
    pub fn satisfied_hard_constraints(&self) -> usize {
        self.hard_states
            .iter()
            .filter(|s| **s == MonitorState::Satisfied)
            .count()
    }

    /// Check if all hard constraints are satisfied or pending (not violated).
    pub fn all_hard_constraints_ok(&self) -> bool {
        self.hard_states
            .iter()
            .all(|s| *s == MonitorState::Satisfied || *s == MonitorState::Pending)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trajectory_policy_new() {
        let policy = TrajectoryPolicy::new(&[], &[]);
        assert_eq!(policy.hard_states.len(), 0);
        assert_eq!(policy.soft_states.len(), 0);
    }
}
