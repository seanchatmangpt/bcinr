//! Design-for-Combinatorial-Maximalism application contracts.
//!
//! This module turns the embedded planning facade into a compiler-shaped
//! application boundary. It contains value objects and contracts only: no
//! command handler is invoked and no external side effect is performed here.

#![cfg(feature = "mfw-planner")]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{
    ActionInvocation, CognitiveExecutionStanding, EmbeddedWorkflow, TypedWorkflowPlan,
    VerifiedWorkflowPlan,
};

include!("workflow_cmd/identity_domain.rs");
include!("workflow_cmd/observation_goal.rs");
include!("workflow_cmd/planning_transport.rs");
include!("workflow_cmd/binding_policy.rs");
include!("workflow_cmd/dispatch.rs");
include!("workflow_cmd/cursor.rs");
include!("workflow_cmd/receipts.rs");
include!("workflow_cmd/residual.rs");
include!("workflow_cmd/passes.rs");
