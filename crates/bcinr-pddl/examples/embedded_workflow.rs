use std::borrow::Cow;

use bcinr_pddl::prelude::*;

const DOMAIN: &str = "(define (domain fulfillment)
  (:requirements :strips)
  (:predicates (paid) (reserved) (customer-notified))
  (:action reserve-inventory
    :parameters ()
    :precondition (paid)
    :effect (reserved))
  (:action notify-customer
    :parameters ()
    :precondition (paid)
    :effect (customer-notified)))";

struct Order {
    id: u64,
    paid: bool,
}

impl WorkflowProblem for Order {
    fn to_pddl_problem(&self) -> Cow<'_, str> {
        Cow::Owned(format!(
            "(define (problem order-{id})
              (:domain fulfillment)
              (:init {paid})
              (:goal (and (reserved) (customer-notified))))",
            id = self.id,
            paid = if self.paid { "(paid)" } else { "" },
        ))
    }
}

#[derive(Debug)]
enum Command {
    ReserveInventory,
    NotifyCustomer,
}

impl TryFrom<ActionInvocation> for Command {
    type Error = String;

    fn try_from(action: ActionInvocation) -> Result<Self, Self::Error> {
        match action.name.as_str() {
            "reserve-inventory" => Ok(Self::ReserveInventory),
            "notify-customer" => Ok(Self::NotifyCustomer),
            _ => Err(format!("no Rust command is bound to {}", action.label)),
        }
    }
}

fn main() {
    // The stable planning domain lives inside the service instead of behind a
    // planner daemon, workflow SaaS API, or BPMN deployment boundary.
    let mut fulfillment = EmbeddedWorkflow::new(DOMAIN);
    let order = Order { id: 42, paid: true };

    // Application state is projected into a problem, planned, executed through
    // POWL, and receipt-verified before any command reaches application code.
    let verified = fulfillment.plan(&order).expect("workflow should be admitted");
    let commands = verified
        .bind::<Command>()
        .expect("every planner action must have a Rust binding");

    println!("standing: {:?}", commands.standing);
    println!("execution root: {}", commands.execution_root);
    for batch in commands.batches() {
        println!("tick {}: {:?}", batch.tick, batch.actions);
        // The host application chooses how to actuate this batch: transaction,
        // queue, actor mailbox, async task group, saga, or command broker.
    }
}
