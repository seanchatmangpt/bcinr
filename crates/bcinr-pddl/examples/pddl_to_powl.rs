use bcinr_pddl::PddlPowlRuntime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let domain = "(define (domain demo)
      (:requirements :strips)
      (:predicates (ready) (left) (right))
      (:action make-left :parameters () :precondition (ready) :effect (left))
      (:action make-right :parameters () :precondition (ready) :effect (right)))";
    let problem = "(define (problem demo-one)
      (:domain demo)
      (:init (ready))
      (:goal (and (left) (right))))";

    let mut runtime = PddlPowlRuntime::default();
    let execution = runtime.execute(domain, problem)?;
    execution.verify()?;

    for (tick, actions) in execution.execution_batches()?.iter().enumerate() {
        println!("tick {tick}: {}", actions.join(", "));
    }
    println!("receipt: {}", execution.state_receipt.chain_root);
    Ok(())
}
