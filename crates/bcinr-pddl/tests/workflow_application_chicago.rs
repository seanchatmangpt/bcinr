//! Chicago TDD coverage for the planning-native application facade.
//!
//! The test drives real planning, POWL verification, binding, policy, adapter
//! projection, broker admission, cursor progression, effect evidence, and receipts.

#![cfg(feature = "mfw-planner")]

use std::borrow::Cow;

use bcinr_pddl::prelude::*;
use serde::Serialize;

const DOMAIN: &str = r#"
(define (domain fulfillment-app)
  (:requirements :strips)
  (:predicates (paid) (reserved) (notified))
  (:action reserve-inventory
    :parameters ()
    :precondition (paid)
    :effect (reserved))
  (:action notify-customer
    :parameters ()
    :precondition (paid)
    :effect (notified)))
"#;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct OrderState {
    order_id: u64,
    paid: bool,
}

impl WorkflowProblem for OrderState {
    fn to_pddl_problem(&self) -> Cow<'_, str> {
        let paid = if self.paid { "(paid)" } else { "" };
        Cow::Owned(format!(
            "(define (problem order-{id}) (:domain fulfillment-app) (:init {paid}) (:goal (and (reserved) (notified))))",
            id = self.order_id,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
enum Command {
    ReserveInventory,
    NotifyCustomer,
}

#[derive(Debug, Clone, Copy)]
struct CommandBinding;

impl ActionBinding for CommandBinding {
    type Command = Command;
    type Error = String;

    fn binding_name(&self) -> &str {
        "fulfillment-app-command-v1"
    }

    fn supported_actions(&self) -> Vec<&str> {
        vec!["reserve-inventory", "notify-customer"]
    }

    fn bind(&self, action: &ActionInvocation) -> Result<Self::Command, Self::Error> {
        match (action.name.as_str(), action.arguments.as_slice()) {
            ("reserve-inventory", []) => Ok(Command::ReserveInventory),
            ("notify-customer", []) => Ok(Command::NotifyCustomer),
            _ => Err(format!("unbound action: {}", action.label)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct PermitTenant;

impl PolicyIdentity for PermitTenant {
    fn root(&self) -> PolicySetRoot {
        PolicySetRoot::hash(b"permit-tenant:acme:v1")
    }
}

impl Policy<TypedWorkflowPlan<Command>, &'static str> for PermitTenant {
    type Evidence = &'static str;
    type Refusal = &'static str;

    fn evaluate(
        &self,
        _input: &TypedWorkflowPlan<Command>,
        tenant: &&'static str,
    ) -> PolicyDecision<Self::Evidence, Self::Refusal> {
        if *tenant == "acme" {
            PolicyDecision::Admit("tenant-admitted")
        } else {
            PolicyDecision::Refuse("tenant-refused")
        }
    }
}

chicago_tdd_tools::test!(
    application_compiles_projects_and_receipts_one_behavioral_flow,
    {
        let workflow = EmbeddedWorkflow::new(DOMAIN).expect("resident domain should install");
        let mut application = WorkflowApplication::new(workflow, CommandBinding)
            .expect("command schema should validate");
        WorkflowAssertions::binding_complete(
            &application
                .bindings()
                .coverage(["reserve-inventory", "notify-customer"]),
        );

        let state = OrderState {
            order_id: 42,
            paid: true,
        };
        let observation = ObservationSnapshot::manufacture(
            LogicalTime(10),
            SourceVersion("orders:42:v7".to_string()),
            state.clone(),
        )
        .expect("observation should canonicalize");
        let goal = GoalEnvelope::manufacture(
            GoalExpr::<String, i64>::All(vec![
                GoalExpr::Atom("reserved".to_string()),
                GoalExpr::Atom("notified".to_string()),
            ]),
            GoalPriority(100),
            None,
            GoalPolicy::Hard,
        )
        .expect("goal should canonicalize");

        let prepared = application
            .compile(
                &state,
                &observation,
                &goal,
                PlanningBounds::interactive(),
                SearchPolicyRoot::hash(b"deterministic-first-valid:v1"),
            )
            .expect("application should compile verified native commands");
        let authorized = prepared
            .authorize_and_propose(
                &PermitTenant,
                &"acme",
                IdempotencyKey::new("order-42:generation-0").expect("idempotency key"),
            )
            .expect("policy should admit the compiled commands");
        assert_eq!(authorized.evidence(), &"tenant-admitted");
        assert_eq!(authorized.proposal().commands().len(), 2);

        let task_batches = TaskGroupAdapter
            .project(authorized.proposal())
            .expect("task projection should be deterministic");
        let task_commands = task_batches
            .iter()
            .map(|batch| batch.commands().len())
            .sum::<usize>();
        assert_eq!(task_commands, 2);
        assert!(task_batches
            .iter()
            .all(|batch| batch.dispatch_root() == authorized.proposal().root()));

        let outbox = OutboxAdapter::new("orders.fulfillment").expect("outbox destination");
        let records = outbox
            .project(authorized.proposal())
            .expect("outbox projection should succeed");
        assert_eq!(records.len(), 2);
        assert_ne!(records[0].message_key(), records[1].message_key());

        let approval = ApprovalAdapter::new("fulfillment commands require review")
            .expect("approval reason")
            .project(authorized.proposal())
            .expect("approval projection should succeed");
        assert_eq!(approval.commands().len(), 2);
        assert_eq!(approval.plan_root(), prepared.envelope().plan_root());

        let mut broker = RecordingBroker::default();
        let admission = broker
            .admit_batch(authorized.proposal())
            .expect("broker should admit the first idempotency key");
        let mut cursor = WorkflowCursor::from_proposal(authorized.proposal());
        cursor
            .record_admission(&admission)
            .expect("cursor should accept matching broker admission");
        let mut observer = RecordingEffectObserver::default();
        let effects = complete_ready_tick(authorized.proposal(), &mut cursor, &mut observer)
            .expect("admitted commands should receive simulated effect evidence");
        assert_eq!(effects.len(), 2);
        WorkflowAssertions::cursor_complete(&cursor);

        let receipts =
            scenario_receipt_chain(&prepared, &authorized, &admission, &cursor, &effects);
        WorkflowAssertions::receipt_chain_valid(&receipts);
        assert_eq!(receipts.records().len(), 11);
    }
);

chicago_tdd_tools::test!(policy_refusal_never_manufactures_dispatch_authority, {
    let workflow = EmbeddedWorkflow::new(DOMAIN).expect("resident domain should install");
    let mut application =
        WorkflowApplication::new(workflow, CommandBinding).expect("binding schema");
    let state = OrderState {
        order_id: 9,
        paid: true,
    };
    let observation = ObservationSnapshot::manufacture(
        LogicalTime(1),
        SourceVersion("orders:9:v1".to_string()),
        state.clone(),
    )
    .expect("observation");
    let goal = GoalEnvelope::manufacture(
        GoalExpr::<String, i64>::Atom("reserved".to_string()),
        GoalPriority(1),
        None,
        GoalPolicy::Hard,
    )
    .expect("goal");
    let prepared = application
        .compile(
            &state,
            &observation,
            &goal,
            PlanningBounds::interactive(),
            SearchPolicyRoot::hash(b"deterministic-first-valid:v1"),
        )
        .expect("planning should succeed before policy");
    assert!(matches!(
        prepared.authorize_and_propose(
            &PermitTenant,
            &"other-tenant",
            IdempotencyKey::new("order-9:generation-0").unwrap(),
        ),
        Err(AuthorizationProposalError::Policy("tenant-refused"))
    ));
});
