//! Narrow downstream prelude for PDDL → POWL cognitive composition.

#![cfg(feature = "mfw-planner")]

pub use crate::{
    execute_cognitive_pddl, execute_cognitive_task, CognitiveExecutionStanding,
    CognitivePddlConfig, CognitivePddlError, CognitivePddlExecution,
    CognitivePddlExecutionSummary, CognitivePddlRuntime, ExactCognitiveBounds, OwnedPddlTask,
    PddlPowlBatch, PddlTask,
};
