//! Slow-rail DTOs for downstream PDDL → POWL consumers.
//!
//! These types are serialization and integration utilities. They are not part
//! of the authoritative branchless scheduler call graph and perform no external
//! actuation.

#![cfg(feature = "mfw-planner")]

use serde::{Deserialize, Serialize};

use crate::{PddlPowlError, PddlPowlExecution};

/// One admitted simultaneous firing batch from the POWL v2 scheduler.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PddlPowlBatch {
    pub tick: u32,
    pub fired_mask: u64,
    pub actions: Vec<String>,
}

/// Portable receipt-and-result view that avoids exposing internal planner IR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PddlPowlExecutionSummary {
    pub version: u16,
    pub planning_receipt_root: String,
    pub projection_receipt_root: String,
    pub powl_execution_root: String,
    pub state_execution_root: String,
    pub batches: Vec<PddlPowlBatch>,
    pub final_state: Vec<String>,
    pub goal_reached: bool,
    pub cache_hit: bool,
}

impl PddlPowlExecutionSummary {
    /// Serialize the portable summary without serializing internal graph types.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Serialize the portable summary for human-facing logs and fixtures.
    pub fn to_pretty_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl PddlPowlExecution {
    /// Return typed scheduler batches for dispatch adapters and UIs.
    pub fn batches(&self) -> Result<Vec<PddlPowlBatch>, PddlPowlError> {
        self.execution_batches()?
            .into_iter()
            .zip(self.powl_receipt.fired_masks.iter().copied())
            .enumerate()
            .map(|(tick, (actions, fired_mask))| {
                Ok(PddlPowlBatch {
                    tick: tick as u32,
                    fired_mask,
                    actions,
                })
            })
            .collect()
    }

    /// Manufacture a stable downstream DTO after verifying both receipt rails.
    pub fn summary(&self) -> Result<PddlPowlExecutionSummary, PddlPowlError> {
        self.verify()?;
        Ok(PddlPowlExecutionSummary {
            version: 1,
            planning_receipt_root: self.workflow.planning_receipt.hash.to_string(),
            projection_receipt_root: self.workflow.projection_receipt.hash.to_string(),
            powl_execution_root: self.powl_receipt.chain_root.clone(),
            state_execution_root: self.state_receipt.chain_root.clone(),
            batches: self.batches()?,
            final_state: self.final_state_labels(),
            goal_reached: self.state_receipt.goal_reached,
            cache_hit: self.workflow.cache_hit,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::execute_pddl_to_powl;

    #[test]
    fn summary_is_json_serializable_and_receipt_complete() {
        let execution = execute_pddl_to_powl(
            "(define (domain d) (:requirements :strips) (:predicates (p)) \
             (:action make-p :parameters () :precondition () :effect (p)))",
            "(define (problem p) (:domain d) (:init) (:goal (p)))",
        )
        .unwrap();
        let summary = execution.summary().unwrap();
        let json = summary.to_json().unwrap();
        assert!(json.contains("state_execution_root"));
        assert_eq!(summary.batches[0].actions, vec!["make-p"]);
        assert!(summary.goal_reached);
    }
}
