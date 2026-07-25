use bcinr_pddl::execute_cognitive_pddl;

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

    let execution = execute_cognitive_pddl(domain, problem)?;
    execution.verify()?;

    println!("standing: {:?}", execution.standing());
    for batch in execution.batches()? {
        if !batch.actions.is_empty() {
            println!("tick {}: {}", batch.tick, batch.actions.join(", "));
        }
    }
    println!("receipt: {}", execution.execution_root());
    Ok(())
}
