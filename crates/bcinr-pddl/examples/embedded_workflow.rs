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
        let mut problem =
            StripsProblemBuilder::new(format!("order-{}", self.id), "fulfillment")
                .expect("application identifiers must be valid PDDL symbols");
        if self.paid {
            problem
                .add_nullary_fact("paid")
                .expect("domain predicates are compile-time constants");
        }
        problem
            .add_nullary_goal("reserved")
            .expect("domain predicates are compile-time constants")
            .add_nullary_goal("customer-notified")
            .expect("domain predicates are compile-time constants");
        Cow::Owned(
            problem
                .build()
                .expect("the application always supplies a goal")
                .into_string(),
        )
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
    // The stable planning domain is validated and installed inside the service
    // instead of behind a planner daemon, workflow SaaS API, or BPMN boundary.
    let mut fulfillment =
        EmbeddedWorkflow::new(DOMAIN).expect("embedded planning domain must parse");
    let order = Order { id: 42, paid: true };

    // Application state is projected into a canonical problem, planned,
    // executed through POWL, and receipt-verified before any command reaches
    // application code.
    let verified = fulfillment.plan(&order).expect("workflow should be admitted");
    let commands = verified
        .bind::<Command>()
        .expect("every planner action must have a Rust binding");

    println!("domain root: {}", fulfillment.domain_source_root());
    println!("standing: {:?}", commands.standing());
    println!("execution root: {}", commands.execution_root());
    for batch in commands.batches() {
        println!("tick {}: {:?}", batch.tick(), batch.actions());
        // The host application chooses how to actuate this batch: transaction,
        // queue, actor mailbox, async task group, saga, or command broker.
    }
}
