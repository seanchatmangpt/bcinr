//! Input carriers and convenience execution helpers for downstream consumers.

#![cfg(feature = "mfw-planner")]

use serde::{Deserialize, Serialize};

use crate::{
    CognitivePddlError, CognitivePddlExecution, CognitivePddlRuntime,
};

/// Borrowed PDDL domain/problem pair for zero-copy request routing.
#[derive(Debug, Clone, Copy)]
pub struct PddlTask<'a> {
    pub domain: &'a str,
    pub problem: &'a str,
}

impl<'a> PddlTask<'a> {
    pub const fn new(domain: &'a str, problem: &'a str) -> Self {
        Self { domain, problem }
    }

    pub fn to_owned(self) -> OwnedPddlTask {
        OwnedPddlTask {
            domain: self.domain.to_string(),
            problem: self.problem.to_string(),
        }
    }
}

/// Owned PDDL request suitable for queues, persistence, and connector DTOs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnedPddlTask {
    pub domain: String,
    pub problem: String,
}

impl OwnedPddlTask {
    pub fn new(domain: impl Into<String>, problem: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            problem: problem.into(),
        }
    }

    pub fn as_task(&self) -> PddlTask<'_> {
        PddlTask::new(&self.domain, &self.problem)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl CognitivePddlRuntime {
    /// Execute a borrowed task while preserving this runtime's standing cache.
    pub fn execute_task(
        &mut self,
        task: PddlTask<'_>,
    ) -> Result<CognitivePddlExecution, CognitivePddlError> {
        self.execute(task.domain, task.problem)
    }

    /// Execute an owned task without copying its PDDL strings.
    pub fn execute_owned_task(
        &mut self,
        task: &OwnedPddlTask,
    ) -> Result<CognitivePddlExecution, CognitivePddlError> {
        self.execute_task(task.as_task())
    }
}

/// Stateless convenience function over a borrowed task carrier.
pub fn execute_cognitive_task(
    task: PddlTask<'_>,
) -> Result<CognitivePddlExecution, CognitivePddlError> {
    CognitivePddlRuntime::default().execute_task(task)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_task_json_roundtrip_and_execution() {
        let task = OwnedPddlTask::new(
            "(define (domain d) (:requirements :strips) (:predicates (done)) \
             (:action finish :parameters () :precondition () :effect (done)))",
            "(define (problem p) (:domain d) (:init) (:goal (done)))",
        );
        let decoded = OwnedPddlTask::from_json(&task.to_json().unwrap()).unwrap();
        assert_eq!(decoded, task);
        let execution = execute_cognitive_task(decoded.as_task()).unwrap();
        execution.verify().unwrap();
    }
}
