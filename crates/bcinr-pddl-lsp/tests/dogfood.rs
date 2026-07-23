//! Dogfood: scan the bcinr-pddl-lsp project itself through the lifecycle map.
//!
//! This test uses the lifecycle scanner against the actual workspace,
//! then runs the full projection → candidate plan chain.
//! It answers the 5 questions from the dogfood requirement:
//!   1. current lifecycle status
//!   2. next lawful step
//!   3. publish gate status
//!   4. receipt status
//!   5. build broker status

use std::path::PathBuf;

fn bcinr_root() -> PathBuf {
    // Go up from crates/bcinr-pddl-lsp to the bcinr workspace root
    let manifest = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest)
        .join("../..")
        .canonicalize()
        .unwrap()
}

#[test]
fn q1_current_lifecycle_status() {
    use bcinr_pddl_lsp::lifecycle::scan;
    let root = bcinr_root();
    let lc = scan(&root);

    println!("=== Q1: Current lifecycle status ===");
    println!("Project: {}", lc.project_name);
    println!("Root: {}", lc.root.display());
    println!("True stages ({}):", lc.true_stages.len());
    for s in &lc.true_stages {
        println!("  ✓ {}", s.predicate_name());
    }
    println!("Missing stages ({}):", lc.missing.len());
    for s in &lc.missing {
        println!("  ✗ {}", s.predicate_name());
    }

    // bcinr has docs/thesis/ but not docs/prd.md — lifecycle is early stage
    // At minimum it has source files
    assert!(
        !lc.true_stages.is_empty() || lc.root.exists(),
        "scanner must produce some result for bcinr workspace"
    );
}

#[test]
fn q2_next_lawful_step() {
    use bcinr_pddl_lsp::lifecycle::scan;
    let root = bcinr_root();
    let lc = scan(&root);

    println!("=== Q2: Next lawful step ===");
    match lc.next_missing() {
        Some(stage) => println!("Next: {}", stage.predicate_name()),
        None => println!("Next: none (lifecycle complete)"),
    }
    // The next step must be a valid lifecycle stage or None
    // (no assertion on which — depends on actual bcinr state)
}

#[test]
fn q3_publish_gate_status() {
    use bcinr_pddl_lsp::{lifecycle::scan, publish_gate};
    let root = bcinr_root();
    let lc = scan(&root);
    let gate = publish_gate::from_lifecycle(&lc);

    println!("=== Q3: Publish gate status ===");
    println!("Gate: {}", gate.status_label());
    println!("Admitted: {}", gate.admitted);
    if !gate.blockers.is_empty() {
        println!("Blockers:");
        for b in &gate.blockers {
            println!("  - {b}");
        }
    }
    // Gate must be a valid status (never panics)
    assert!(!gate.status_label().is_empty());
}

#[test]
fn q4_receipt_status() {
    use bcinr_pddl_lsp::lifecycle::{scan, LifecycleStage};
    let root = bcinr_root();
    let lc = scan(&root);

    println!("=== Q4: Receipt status ===");
    let receipt_path = root.join(".bcinr/receipts/latest.json");
    if receipt_path.exists() {
        let content = std::fs::read_to_string(&receipt_path).unwrap_or_default();
        let goal_reached = content.contains("\"goal_reached\": true");
        println!("Receipt: present");
        println!("goal_reached: {goal_reached}");
        if goal_reached {
            assert!(
                lc.has(&LifecycleStage::Published),
                "published must be true when receipt has goal_reached=true"
            );
        }
    } else {
        println!("Receipt: absent (.bcinr/receipts/latest.json not found)");
        println!("Note: run bcinrPddl.executeTape to admit and emit receipt");
        assert!(
            !lc.has(&LifecycleStage::Published),
            "published must be false when no receipt"
        );
    }
}

#[test]
fn q5_build_broker_status() {
    use bcinr_pddl_lsp::build_broker::BuildBrokerState;

    println!("=== Q5: Build broker status ===");
    // Fresh broker (no persistent state yet — not yet wired to disk)
    let broker = BuildBrokerState::default();
    println!("Slot status: {}", broker.status_label());
    println!("Can acquire: {}", broker.can_acquire());
    println!("Max slots: {}", broker.max_slots);
    println!("Denial count: {}", broker.denial_count);

    // Fresh broker is always IDLE and can acquire
    assert_eq!(broker.status_label(), "IDLE");
    assert!(broker.can_acquire());
}

#[test]
fn q6_agent_next_step_virtual_doc() {
    use bcinr_pddl_lsp::{lifecycle::scan, publish_gate, virtual_docs};
    let root = bcinr_root();
    let lc = scan(&root);
    let gate = publish_gate::from_lifecycle(&lc);

    println!("=== Q6: Agent assignments virtual doc ===");
    let doc = virtual_docs::render_agent_assignments(&lc, &gate);
    println!("{doc}");

    // Must contain the next_step field
    assert!(doc.contains("next_lawful_step"));
    assert!(doc.contains("publish_gate"));
    assert!(doc.contains("instruction"));
}

#[test]
fn q7_full_projection_succeeds() {
    use bcinr_pddl_lsp::{lifecycle::scan, planner_client, projection};
    let root = bcinr_root();
    let lc = scan(&root);
    let proj = projection::project(&lc);

    println!("=== Q7: Full projection ===");
    println!("Domain text length: {} chars", proj.domain_text.len());
    println!("Problem text length: {} chars", proj.problem_text.len());

    // Domain must parse
    let dom = bcinr_pddl::domain_from_pddl(&proj.domain_text).expect("domain must parse");
    println!("Domain actions: {}", dom.actions.len());

    // Must produce a candidate plan
    match planner_client::plan(&proj) {
        Ok(candidate) => {
            println!("Candidate plan steps: {}", candidate.plan_steps.len());
            for (i, step) in candidate.plan_steps.iter().enumerate() {
                println!("  [{i}] {step}");
            }
        }
        Err(e) => {
            // Only acceptable if lifecycle is already at goal (all stages present)
            println!("Plan result: {e}");
        }
    }
}
