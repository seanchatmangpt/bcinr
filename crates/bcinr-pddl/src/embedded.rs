//! Application-facing abstractions for embedding planning and workflow inside Rust services.
//!
//! The planner remains a decision manufacturer, not an actuator. This module
//! turns a verified PDDL → POWL execution into typed, serializable application
//! work that the host program can enqueue, transact, supervise, or broker under
//! its own side-effect policy.

#![cfg(feature = "mfw-planner")]

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::{
    domain31_from_pddl, CognitiveExecutionStanding, CognitivePddlConfig, CognitivePddlError,
    CognitivePddlExecution, CognitivePddlExecutionSummary, CognitivePddlRuntime, Pddl8Error,
    PddlBuildError, StripsProblemBuilder,
};

/// Application-owned source of a PDDL problem document.
///
/// Implement this trait for domain state, request DTOs, aggregate roots, or
/// projections. The stable domain remains resident in [`EmbeddedWorkflow`],
/// while each application value manufactures the current problem instance.
pub trait WorkflowProblem {
    fn to_pddl_problem(&self) -> Cow<'_, str>;
}

impl WorkflowProblem for str {
    fn to_pddl_problem(&self) -> Cow<'_, str> {
        Cow::Borrowed(self)
    }
}

impl WorkflowProblem for String {
    fn to_pddl_problem(&self) -> Cow<'_, str> {
        Cow::Borrowed(self.as_str())
    }
}

impl<'a> WorkflowProblem for Cow<'a, str> {
    fn to_pddl_problem(&self) -> Cow<'_, str> {
        Cow::Borrowed(self.as_ref())
    }
}

/// Parsed application-level action invocation.
///
/// The parser accepts the label forms emitted by the planning rails and common
/// connector representations:
///
/// - `reserve(order-7,warehouse-a)`
/// - `(reserve order-7 warehouse-a)`
/// - `reserve order-7 warehouse-a`
/// - `reserve`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionInvocation {
    pub label: String,
    pub name: String,
    pub arguments: Vec<String>,
}

impl ActionInvocation {
    pub fn parse(label: impl Into<String>) -> Result<Self, ActionLabelError> {
        let label = label.into();
        let trimmed = label.trim();
        if trimmed.is_empty() {
            return Err(ActionLabelError::new(label, "action label is empty"));
        }

        let (name, arguments) = if trimmed.starts_with('(') {
            let Some(inner) = trimmed.strip_prefix('(').and_then(|value| value.strip_suffix(')'))
            else {
                return Err(ActionLabelError::new(
                    label,
                    "S-expression action label has unbalanced parentheses",
                ));
            };
            if inner.contains('(') || inner.contains(')') {
                return Err(ActionLabelError::new(
                    label,
                    "nested parentheses are not admitted in an action label",
                ));
            }
            split_whitespace_invocation(inner, &label)?
        } else if let Some(open) = trimmed.find('(') {
            if !trimmed.ends_with(')') || trimmed[..open].contains(')') {
                return Err(ActionLabelError::new(
                    label,
                    "function-style action label has unbalanced parentheses",
                ));
            }
            let name = trimmed[..open].trim();
            let arguments = &trimmed[open + 1..trimmed.len() - 1];
            if name.is_empty() {
                return Err(ActionLabelError::new(label, "action name is empty"));
            }
            if arguments.contains('(') || arguments.contains(')') {
                return Err(ActionLabelError::new(
                    label,
                    "nested parentheses are not admitted in action arguments",
                ));
            }
            let arguments = if arguments.trim().is_empty() {
                Vec::new()
            } else {
                arguments
                    .split(',')
                    .enumerate()
                    .map(|(index, argument)| {
                        let argument = argument.trim();
                        if argument.is_empty() {
                            Err(ActionLabelError::new(
                                label.clone(),
                                format!("action argument {index} is empty"),
                            ))
                        } else {
                            Ok(argument.to_string())
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()?
            };
            (name.to_string(), arguments)
        } else {
            if trimmed.contains(')') {
                return Err(ActionLabelError::new(
                    label,
                    "action label has an unmatched closing parenthesis",
                ));
            }
            split_whitespace_invocation(trimmed, &label)?
        };

        Ok(Self {
            label,
            name,
            arguments,
        })
    }
}

fn split_whitespace_invocation(
    value: &str,
    original: &str,
) -> Result<(String, Vec<String>), ActionLabelError> {
    let mut parts = value.split_whitespace();
    let Some(name) = parts.next() else {
        return Err(ActionLabelError::new(original, "action name is empty"));
    };
    Ok((
        name.to_string(),
        parts.map(ToString::to_string).collect(),
    ))
}

/// Refusal to convert a planner label into an application invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionLabelError {
    pub label: String,
    pub reason: String,
}

impl ActionLabelError {
    fn new(label: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            reason: reason.into(),
        }
    }
}

impl std::fmt::Display for ActionLabelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid workflow action label {:?}: {}", self.label, self.reason)
    }
}

impl std::error::Error for ActionLabelError {}

/// One verified scheduler tick expressed as application-level invocations.
///
/// Construction is private to the verified-plan boundary. Serialization is
/// one-way until a durable replay envelope can re-establish standing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkflowBatch {
    tick: u32,
    fired_mask: u64,
    actions: Vec<ActionInvocation>,
}

impl WorkflowBatch {
    pub fn tick(&self) -> u32 {
        self.tick
    }

    pub fn fired_mask(&self) -> u64 {
        self.fired_mask
    }

    pub fn actions(&self) -> &[ActionInvocation] {
        &self.actions
    }

    pub fn into_actions(self) -> Vec<ActionInvocation> {
        self.actions
    }

    /// Whether this tick contains more than one independently admitted action.
    pub fn is_parallel(&self) -> bool {
        self.actions.len() > 1
    }
}

/// Error at the embedded application boundary.
#[derive(Debug)]
pub enum EmbeddedWorkflowError {
    Planning(CognitivePddlError),
    ProblemBuild(PddlBuildError),
    InvalidActionLabel(ActionLabelError),
}

impl std::fmt::Display for EmbeddedWorkflowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Planning(error) => write!(f, "embedded workflow planning failed: {error}"),
            Self::ProblemBuild(error) => write!(f, "embedded workflow problem build failed: {error}"),
            Self::InvalidActionLabel(error) => write!(f, "embedded workflow binding failed: {error}"),
        }
    }
}

impl std::error::Error for EmbeddedWorkflowError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Planning(error) => Some(error),
            Self::ProblemBuild(error) => Some(error),
            Self::InvalidActionLabel(error) => Some(error),
        }
    }
}

impl From<CognitivePddlError> for EmbeddedWorkflowError {
    fn from(error: CognitivePddlError) -> Self {
        Self::Planning(error)
    }
}

impl From<PddlBuildError> for EmbeddedWorkflowError {
    fn from(error: PddlBuildError) -> Self {
        Self::ProblemBuild(error)
    }
}

impl From<ActionLabelError> for EmbeddedWorkflowError {
    fn from(error: ActionLabelError) -> Self {
        Self::InvalidActionLabel(error)
    }
}

/// A verified plan ready to cross into application-specific command binding.
///
/// Construction is private: callers can only obtain this value after the
/// selected semantic rail and its POWL execution receipt have verified.
pub struct VerifiedWorkflowPlan {
    execution: CognitivePddlExecution,
    batches: Vec<WorkflowBatch>,
}

impl std::fmt::Debug for VerifiedWorkflowPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VerifiedWorkflowPlan")
            .field("standing", &self.standing())
            .field("execution_root", &self.execution_root())
            .field("batches", &self.batches)
            .finish()
    }
}

impl VerifiedWorkflowPlan {
    fn manufacture(execution: CognitivePddlExecution) -> Result<Self, EmbeddedWorkflowError> {
        execution.verify()?;
        let batches = execution
            .batches()?
            .into_iter()
            .map(|batch| {
                let actions = batch
                    .actions
                    .into_iter()
                    .map(ActionInvocation::parse)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(WorkflowBatch {
                    tick: batch.tick,
                    fired_mask: batch.fired_mask,
                    actions,
                })
            })
            .collect::<Result<Vec<_>, ActionLabelError>>()?;
        Ok(Self { execution, batches })
    }

    pub fn standing(&self) -> CognitiveExecutionStanding {
        self.execution.standing()
    }

    /// Root the host application should bind into its own command or actuation receipt.
    pub fn execution_root(&self) -> &str {
        self.execution.execution_root()
    }

    pub fn batches(&self) -> &[WorkflowBatch] {
        &self.batches
    }

    pub fn summary(&self) -> Result<CognitivePddlExecutionSummary, CognitivePddlError> {
        self.execution.summary()
    }

    pub fn into_execution(self) -> CognitivePddlExecution {
        self.execution
    }

    /// Convert planner invocations into an application command enum or DTO.
    ///
    /// This is a pure binding step. It never calls handlers and never mutates
    /// application state.
    pub fn bind<A>(
        &self,
    ) -> Result<TypedWorkflowPlan<A>, <A as TryFrom<ActionInvocation>>::Error>
    where
        A: TryFrom<ActionInvocation>,
    {
        self.map_actions(|action| A::try_from(action.clone()))
    }

    /// Bind actions with a closure when implementing `TryFrom` is inconvenient.
    pub fn map_actions<A, E, F>(&self, mut mapper: F) -> Result<TypedWorkflowPlan<A>, E>
    where
        F: FnMut(&ActionInvocation) -> Result<A, E>,
    {
        let batches = self
            .batches
            .iter()
            .map(|batch| {
                let actions = batch
                    .actions
                    .iter()
                    .map(&mut mapper)
                    .collect::<Result<Vec<_>, E>>()?;
                Ok(TypedWorkflowBatch {
                    tick: batch.tick,
                    fired_mask: batch.fired_mask,
                    actions,
                })
            })
            .collect::<Result<Vec<_>, E>>()?;
        Ok(TypedWorkflowPlan {
            standing: self.standing(),
            execution_root: self.execution_root().to_string(),
            batches,
        })
    }
}

/// A verified workflow whose actions have been converted to host application types.
///
/// Fields are private and this type is not deserializable: receiving a JSON
/// object with the same shape is not sufficient to recreate verified standing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TypedWorkflowPlan<A> {
    standing: CognitiveExecutionStanding,
    execution_root: String,
    batches: Vec<TypedWorkflowBatch<A>>,
}

impl<A> TypedWorkflowPlan<A> {
    pub fn standing(&self) -> CognitiveExecutionStanding {
        self.standing
    }

    pub fn execution_root(&self) -> &str {
        &self.execution_root
    }

    pub fn batches(&self) -> &[TypedWorkflowBatch<A>] {
        &self.batches
    }

    pub fn into_batches(self) -> Vec<TypedWorkflowBatch<A>> {
        self.batches
    }
}

/// One application-ready batch. Actions within a batch may execute concurrently;
/// batches retain their admitted tick order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TypedWorkflowBatch<A> {
    tick: u32,
    fired_mask: u64,
    actions: Vec<A>,
}

impl<A> TypedWorkflowBatch<A> {
    pub fn tick(&self) -> u32 {
        self.tick
    }

    pub fn fired_mask(&self) -> u64 {
        self.fired_mask
    }

    pub fn actions(&self) -> &[A] {
        &self.actions
    }

    pub fn into_actions(self) -> Vec<A> {
        self.actions
    }

    pub fn is_parallel(&self) -> bool {
        self.actions.len() > 1
    }
}

/// Domain-scoped planning runtime embedded directly in a Rust application.
///
/// Construction parses the domain once to fail fast and captures source
/// identity. Repeated problem instances reuse the standing cache inside
/// [`CognitivePddlRuntime`]. Planning rails may still perform their own parsing
/// as part of receipt-producing semantic admission.
pub struct EmbeddedWorkflow {
    domain_pddl: String,
    domain_name: String,
    domain_source_root: String,
    runtime: CognitivePddlRuntime,
}

impl EmbeddedWorkflow {
    /// Validate and install one resident planning domain.
    pub fn new(domain_pddl: impl Into<String>) -> Result<Self, Pddl8Error> {
        Self::with_config(domain_pddl, CognitivePddlConfig::default())
    }

    /// Validate and install one resident planning domain with explicit bounds.
    pub fn with_config(
        domain_pddl: impl Into<String>,
        config: CognitivePddlConfig,
    ) -> Result<Self, Pddl8Error> {
        let domain_pddl = domain_pddl.into();
        let domain = domain31_from_pddl(&domain_pddl)?;
        let domain_source_root = blake3::hash(domain_pddl.as_bytes()).to_hex().to_string();
        Ok(Self {
            domain_pddl,
            domain_name: domain.name,
            domain_source_root,
            runtime: CognitivePddlRuntime::new(config),
        })
    }

    pub fn domain_pddl(&self) -> &str {
        &self.domain_pddl
    }

    pub fn domain_name(&self) -> &str {
        &self.domain_name
    }

    /// BLAKE3 root of the exact installed domain source.
    pub fn domain_source_root(&self) -> &str {
        &self.domain_source_root
    }

    /// Start a validated positive STRIPS/typing problem for this domain.
    pub fn strips_problem(
        &self,
        problem_name: impl Into<String>,
    ) -> Result<StripsProblemBuilder, PddlBuildError> {
        StripsProblemBuilder::new(problem_name, self.domain_name.clone())
    }

    /// Configure, build, and plan a common positive STRIPS/typing problem.
    pub fn plan_strips<F>(
        &mut self,
        problem_name: impl Into<String>,
        configure: F,
    ) -> Result<VerifiedWorkflowPlan, EmbeddedWorkflowError>
    where
        F: FnOnce(&mut StripsProblemBuilder) -> Result<(), PddlBuildError>,
    {
        let mut problem = self.strips_problem(problem_name)?;
        configure(&mut problem)?;
        let problem = problem.build()?;
        self.plan(&problem)
    }

    /// Plan from any application value that can project itself into PDDL.
    pub fn plan<P>(&mut self, problem: &P) -> Result<VerifiedWorkflowPlan, EmbeddedWorkflowError>
    where
        P: WorkflowProblem + ?Sized,
    {
        let problem_pddl = problem.to_pddl_problem();
        self.plan_pddl(problem_pddl.as_ref())
    }

    /// Plan directly from a PDDL problem document.
    pub fn plan_pddl(
        &mut self,
        problem_pddl: &str,
    ) -> Result<VerifiedWorkflowPlan, EmbeddedWorkflowError> {
        let execution = self.runtime.execute(&self.domain_pddl, problem_pddl)?;
        VerifiedWorkflowPlan::manufacture(execution)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Command {
        Finish,
    }

    impl TryFrom<ActionInvocation> for Command {
        type Error = String;

        fn try_from(action: ActionInvocation) -> Result<Self, Self::Error> {
            match (action.name.as_str(), action.arguments.as_slice()) {
                ("finish", []) => Ok(Self::Finish),
                _ => Err(format!("unbound action: {}", action.label)),
            }
        }
    }

    struct JobState;

    impl WorkflowProblem for JobState {
        fn to_pddl_problem(&self) -> Cow<'_, str> {
            Cow::Borrowed(
                "(define (problem job) (:domain jobs) (:init (ready)) (:goal (done)))",
            )
        }
    }

    const JOB_DOMAIN: &str = "(define (domain jobs) (:requirements :strips) \
        (:predicates (ready) (done)) \
        (:action finish :parameters () :precondition (ready) :effect (done)))";

    #[test]
    fn parses_common_action_label_forms() {
        assert_eq!(
            ActionInvocation::parse("move(a,b)").unwrap().arguments,
            vec!["a", "b"]
        );
        assert_eq!(
            ActionInvocation::parse("(move a b)").unwrap().arguments,
            vec!["a", "b"]
        );
        assert_eq!(
            ActionInvocation::parse("move a b").unwrap().arguments,
            vec!["a", "b"]
        );
        assert_eq!(ActionInvocation::parse("finish").unwrap().name, "finish");
        assert!(ActionInvocation::parse("move(a,,b)").is_err());
    }

    #[test]
    fn embedded_runtime_manufactures_verified_typed_work() {
        let mut workflow = EmbeddedWorkflow::new(JOB_DOMAIN).unwrap();
        assert_eq!(workflow.domain_name(), "jobs");
        assert_eq!(workflow.domain_source_root().len(), 64);

        let plan = workflow.plan(&JobState).unwrap();
        assert_eq!(
            plan.standing(),
            CognitiveExecutionStanding::WitnessedConcurrentStrips
        );
        assert!(!plan.execution_root().is_empty());

        let typed = plan.bind::<Command>().unwrap();
        assert_eq!(typed.batches()[0].actions(), &[Command::Finish]);
    }

    #[test]
    fn plan_strips_eliminates_domain_name_duplication() {
        let mut workflow = EmbeddedWorkflow::new(JOB_DOMAIN).unwrap();
        let plan = workflow
            .plan_strips("job-2", |problem| {
                problem
                    .add_nullary_fact("ready")?
                    .add_nullary_goal("done")?;
                Ok(())
            })
            .unwrap();
        assert_eq!(
            plan.standing(),
            CognitiveExecutionStanding::WitnessedConcurrentStrips
        );
    }

    #[test]
    fn invalid_domain_is_refused_at_construction() {
        assert!(EmbeddedWorkflow::new("(define (domain broken)").is_err());
    }
}
