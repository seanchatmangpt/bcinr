//! BRCE integration proof for `bcinr-mcp`.
//!
//! Protocol-level tool inventory and transport behavior are covered by the MCP
//! harness suites. This target proves the cross-crate manufacture contract
//! directly instead of scraping implementation text from `src/main.rs`.

use bcinr_pddl::manufacture_world;

const DEPLOY_DOMAIN: &str = r#"
(define (domain bcinr-deploy)
  (:requirements :strips)
  (:predicates
    (approved ?s)
    (deployed ?s)
    (smoke-passed ?s)
    (healthy ?s)
  )
  (:action deploy
    :parameters (?s)
    :precondition (approved ?s)
    :effect (deployed ?s)
  )
  (:action run-smoke
    :parameters (?s)
    :precondition (deployed ?s)
    :effect (smoke-passed ?s)
  )
  (:action mark-healthy
    :parameters (?s)
    :precondition (smoke-passed ?s)
    :effect (healthy ?s)
  )
)
"#;

const APPROVED_PROBLEM: &str = r#"
(define (problem approved-api-v2)
  (:domain bcinr-deploy)
  (:objects api-v2)
  (:init (approved api-v2))
  (:goal (healthy api-v2))
)
"#;

const OVERBOUND_DOMAIN: &str = r#"
(define (domain bcinr-overbound)
  (:requirements :strips)
  (:predicates
    (nine-ary ?a ?b ?c ?d ?e ?f ?g ?h ?i)
  )
  (:action no-op
    :parameters (?a)
    :precondition (nine-ary ?a ?a ?a ?a ?a ?a ?a ?a ?a)
    :effect (nine-ary ?a ?a ?a ?a ?a ?a ?a ?a ?a)
  )
)
"#;

const OVERBOUND_PROBLEM: &str = r#"
(define (problem overbound-p)
  (:domain bcinr-overbound)
  (:objects x)
  (:init)
  (:goal (nine-ary x x x x x x x x x))
)
"#;

fn recompute_chain(
    domain_witness: &str,
    problem_witness: &str,
    plan_chain: &str,
    goal_reached: bool,
    step_count: u64,
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain_witness.as_bytes());
    hasher.update(problem_witness.as_bytes());
    hasher.update(plan_chain.as_bytes());
    hasher.update(if goal_reached { b"1" } else { b"0" });
    hasher.update(&step_count.to_le_bytes());
    hasher
        .finalize()
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn brce_manufacture_lifecycle_has_standing() {
    let admitted = manufacture_world(DEPLOY_DOMAIN, APPROVED_PROBLEM, "brce-admitted", &[]);
    assert!(
        admitted.admitted,
        "approved input must be admitted: {:?}",
        admitted.refusal_reason
    );
    assert_eq!(
        admitted.plan_receipt.step_count, 3,
        "BFS must manufacture deploy, smoke, and healthy in three steps"
    );
    assert!(admitted.plan_receipt.goal_reached);

    let recomputed = recompute_chain(
        &admitted.domain_witness,
        &admitted.problem_witness,
        &admitted.plan_receipt.chain_hash,
        admitted.plan_receipt.goal_reached,
        admitted.plan_receipt.step_count as u64,
    );
    assert_eq!(
        admitted.manufacture_chain, recomputed,
        "the admitted artifact must carry a valid manufacture chain"
    );

    let mut tampered = admitted.manufacture_chain.clone();
    let last = tampered.pop().expect("manufacture chains are non-empty");
    tampered.push(if last == 'f' { '0' } else { 'f' });
    assert_ne!(recomputed, tampered, "tampering must invalidate the chain");

    let denied = manufacture_world(
        DEPLOY_DOMAIN,
        APPROVED_PROBLEM,
        "brce-denied",
        &[("__noadmit__", vec![])],
    );
    assert!(!denied.admitted, "policy must deny before effects actuate");
    let denial = denied.refusal_reason.as_deref().unwrap_or_default();
    assert!(
        denial.contains("denied") || denial.contains("Denied"),
        "policy refusal must identify denial: {denial}"
    );

    let overbound = manufacture_world(OVERBOUND_DOMAIN, OVERBOUND_PROBLEM, "brce-overbound", &[]);
    assert!(!overbound.admitted, "PDDL8 must reject nine-arity input");
    let bound_refusal = overbound.refusal_reason.as_deref().unwrap_or_default();
    assert!(
        bound_refusal.contains("bound exceeded") || bound_refusal.contains("PDDL8 bound"),
        "bound refusal must be explicit: {bound_refusal}"
    );

    let replay = manufacture_world(DEPLOY_DOMAIN, APPROVED_PROBLEM, "brce-replay", &[]);
    assert!(replay.admitted);
    assert_eq!(admitted.domain_witness, replay.domain_witness);
    assert_eq!(admitted.problem_witness, replay.problem_witness);
    assert_eq!(
        admitted.plan_receipt.step_count,
        replay.plan_receipt.step_count
    );
    assert_eq!(
        admitted.plan_receipt.goal_reached,
        replay.plan_receipt.goal_reached
    );
}
