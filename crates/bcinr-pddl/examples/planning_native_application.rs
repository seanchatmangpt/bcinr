use std::borrow::Cow;
use std::error::Error;

use bcinr_pddl::prelude::*;
use serde::Serialize;

const DOMAIN: &str = r#"
(define (domain deployment)
  (:requirements :strips)
  (:predicates (built) (deployed) (announced))
  (:action deploy
    :parameters ()
    :precondition (built)
    :effect (deployed))
  (:action announce
    :parameters ()
    :precondition (built)
    :effect (announced)))
"#;

#[derive(Debug, Clone, Serialize)]
struct ReleaseState {
    version: String,
    built: bool,
}

fn render_goal(goal: &GoalExpr<String, i64>) -> String {
    match goal {
        GoalExpr::Atom(atom) => format!("({atom})"),
        GoalExpr::All(goals) => format!(
            "(and {})",
            goals.iter().map(render_goal).collect::<Vec<_>>().join(" ")
        ),
        _ => "(and (deployed) (announced))".to_string(),
    }
}

impl GoalDirectedWorkflowProblem<GoalExpr<String, i64>> for ReleaseState {
    fn to_pddl_problem_for_goal<'a>(
        &'a self,
        goal: &'a GoalExpr<String, i64>,
    ) -> Cow<'a, str> {
        let built = if self.built { "(built)" } else { "" };
        Cow::Owned(format!(
            "(define (problem release-{version}) (:domain deployment) (:init {built}) (:goal {goal}))",
            version = self.version,
            goal = render_goal(goal),
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
enum ReleaseCommand {
    Deploy,
    Announce,
}

struct ReleaseBinding;

impl ActionBinding for ReleaseBinding {
    type Command = ReleaseCommand;
    type Error = String;

    fn binding_name(&self) -> &str {
        "release-command-v1"
    }

    fn supported_actions(&self) -> Vec<&str> {
        vec!["deploy", "announce"]
    }

    fn bind(&self, action: &ActionInvocation) -> Result<Self::Command, Self::Error> {
        match (action.name.as_str(), action.arguments.as_slice()) {
            ("deploy", []) => Ok(ReleaseCommand::Deploy),
            ("announce", []) => Ok(ReleaseCommand::Announce),
            _ => Err(format!("unsupported release action: {}", action.label)),
        }
    }
}

struct PermitRelease;

impl PolicyIdentity for PermitRelease {
    fn root(&self) -> PolicySetRoot {
        PolicySetRoot::hash(b"permit-release:v1")
    }
}

impl Policy<TypedWorkflowPlan<ReleaseCommand>, ()> for PermitRelease {
    type Evidence = &'static str;
    type Refusal = &'static str;

    fn evaluate(
        &self,
        _input: &TypedWorkflowPlan<ReleaseCommand>,
        _context: &(),
    ) -> PolicyDecision<Self::Evidence, Self::Refusal> {
        PolicyDecision::Admit("release-policy-admitted")
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let workflow = EmbeddedWorkflow::new(DOMAIN)?;
    let mut application = WorkflowApplication::new(workflow, ReleaseBinding)?;
    let state = ReleaseState {
        version: "26.7.24".to_string(),
        built: true,
    };
    let observation = ObservationSnapshot::manufacture(
        LogicalTime(1),
        SourceVersion("release-store:1".to_string()),
        state,
    )?;
    let goal = GoalEnvelope::manufacture(
        GoalExpr::<String, i64>::All(vec![
            GoalExpr::Atom("deployed".to_string()),
            GoalExpr::Atom("announced".to_string()),
        ]),
        GoalPriority(100),
        None,
        GoalPolicy::Hard,
    )?;
    let prepared = application.compile_goal_directed(
        &observation,
        &goal,
        PlanningBounds::interactive(),
        SearchPolicyRoot::hash(b"deterministic-first-valid:v1"),
    )?;
    let authorized = prepared.authorize_and_propose(
        &PermitRelease,
        &(),
        IdempotencyKey::new("release-26.7.24:generation-0")?,
    )?;

    let task_batches = TaskGroupAdapter.project(authorized.proposal())?;
    let outbox = OutboxAdapter::new("release.commands")?.project(authorized.proposal())?;

    println!("plan_root={}", prepared.envelope().plan_root());
    println!("dispatch_root={}", authorized.proposal().root());
    println!("task_batches={}", task_batches.len());
    println!("outbox_records={}", outbox.len());
    Ok(())
}
