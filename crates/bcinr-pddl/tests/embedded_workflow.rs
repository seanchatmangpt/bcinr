#![cfg(feature = "mfw-planner")]

use std::borrow::Cow;

use bcinr_pddl::{
    ActionInvocation, CognitiveExecutionStanding, EmbeddedWorkflow, WorkflowProblem,
};

const ORDER_DOMAIN: &str = "(define (domain order-service)
  (:requirements :strips)
  (:predicates
    (payment-authorized)
    (inventory-reserved)
    (confirmation-sent))
  (:action reserve-inventory
    :parameters ()
    :precondition (payment-authorized)
    :effect (inventory-reserved))
  (:action send-confirmation
    :parameters ()
    :precondition (payment-authorized)
    :effect (confirmation-sent)))";

struct OrderState {
    order_id: String,
    payment_authorized: bool,
}

impl WorkflowProblem for OrderState {
    fn to_pddl_problem(&self) -> Cow<'_, str> {
        let init = if self.payment_authorized {
            "(payment-authorized)"
        } else {
            ""
        };
        Cow::Owned(format!(
            "(define (problem order-{id})
              (:domain order-service)
              (:init {init})
              (:goal (and (inventory-reserved) (confirmation-sent))))",
            id = self.order_id,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum OrderCommand {
    ReserveInventory,
    SendConfirmation,
}

impl TryFrom<ActionInvocation> for OrderCommand {
    type Error = String;

    fn try_from(action: ActionInvocation) -> Result<Self, Self::Error> {
        match (action.name.as_str(), action.arguments.as_slice()) {
            ("reserve-inventory", []) => Ok(Self::ReserveInventory),
            ("send-confirmation", []) => Ok(Self::SendConfirmation),
            _ => Err(format!("unbound planner action: {}", action.label)),
        }
    }
}

#[test]
fn rust_application_projects_state_plans_and_binds_typed_commands() {
    let state = OrderState {
        order_id: "42".to_string(),
        payment_authorized: true,
    };
    let mut workflow = EmbeddedWorkflow::new(ORDER_DOMAIN).unwrap();
    assert_eq!(workflow.domain_name(), "order-service");
    assert_eq!(workflow.domain_source_root().len(), 64);

    let verified = workflow.plan(&state).unwrap();
    assert_eq!(
        verified.standing(),
        CognitiveExecutionStanding::WitnessedConcurrentStrips
    );
    assert!(!verified.execution_root().is_empty());

    let typed = verified.bind::<OrderCommand>().unwrap();
    let parallel = typed
        .batches()
        .iter()
        .find(|batch| batch.is_parallel())
        .expect("independent application work should share one admitted tick");
    assert!(parallel.actions().contains(&OrderCommand::ReserveInventory));
    assert!(parallel.actions().contains(&OrderCommand::SendConfirmation));
}
