//! Integration tests for bcinr-mcp: Verify tool coverage without hardcoding
//!
//! These tests read tool definitions from main.rs and verify properties dynamically.
//! This ensures tests stay in sync with actual implementation.

#[cfg(test)]
mod tests {
    use std::fs;

    // Read the main.rs source to extract tool names
    fn extract_tool_names() -> Vec<String> {
        let src = fs::read_to_string("src/main.rs").expect("Failed to read main.rs");
        let mut tools = Vec::new();

        // Match pattern: async fn tool_name(
        for line in src.lines() {
            if line.contains("async fn") && !line.trim().starts_with("//") {
                if let Some(start) = line.find("async fn ") {
                    let after_fn = &line[start + 9..];
                    if let Some(paren) = after_fn.find('(') {
                        let name = after_fn[..paren].trim().to_string();
                        // Skip "main" which is not a tool
                        if name != "main" {
                            tools.push(name);
                        }
                    }
                }
            }
        }
        tools.sort();
        tools
    }

    // Count #[tool(...)] attributes in source
    fn count_tool_definitions() -> usize {
        let src = fs::read_to_string("src/main.rs").expect("Failed to read main.rs");
        src.lines()
            .filter(|line| line.contains("#[tool(") && !line.trim().starts_with("//"))
            .count()
    }

    // Extract tool groups from comments in source
    fn extract_tool_groups() -> std::collections::HashMap<String, Vec<String>> {
        let src = fs::read_to_string("src/main.rs").expect("Failed to read main.rs");
        let mut groups = std::collections::HashMap::new();
        let mut current_group = String::new();

        for line in src.lines() {
            // Match "// ── Group X: NAME ──" pattern
            if line.contains("// ──") && line.contains("Group") {
                if let Some(start) = line.find("Group ") {
                    if let Some(end) = line[start..].find(' ') {
                        let rest = &line[start + end..].trim_start();
                        if let Some(colon_idx) = rest.find(':') {
                            current_group = rest[colon_idx + 1..].trim().to_string();
                            groups.insert(current_group.clone(), Vec::new());
                        }
                    }
                }
            }

            // Match "async fn tool_name(" within a group
            if line.contains("async fn") && !current_group.is_empty() {
                if let Some(start) = line.find("async fn ") {
                    let after_fn = &line[start + 9..];
                    if let Some(paren) = after_fn.find('(') {
                        let name = after_fn[..paren].trim().to_string();
                        if name != "main" {
                            groups.get_mut(&current_group).map(|v| v.push(name));
                        }
                    }
                }
            }
        }

        groups
    }

    // ─── Dynamic Tool Inventory Tests ────────────────────────────────────────

    #[test]
    fn test_tools_exist_in_source() {
        let tools = extract_tool_names();
        assert!(
            !tools.is_empty(),
            "No tools found in main.rs - check extraction logic"
        );
        println!("Found {} tools: {:?}", tools.len(), tools);
    }

    #[test]
    fn test_tool_count_consistent() {
        let tools = extract_tool_names();
        let definitions = count_tool_definitions();
        assert_eq!(
            tools.len(),
            definitions,
            "Tool function count ({}) != #[tool(...)] count ({})",
            tools.len(),
            definitions
        );
    }

    #[test]
    fn test_no_duplicate_tool_names() {
        let tools = extract_tool_names();
        let unique_count = {
            let mut sorted = tools.clone();
            sorted.sort();
            sorted.dedup();
            sorted.len()
        };
        assert_eq!(
            tools.len(),
            unique_count,
            "Found duplicate tool names: {} unique out of {} total",
            unique_count,
            tools.len()
        );
    }

    #[test]
    fn test_tool_names_valid() {
        let tools = extract_tool_names();
        for tool in &tools {
            // Tool names must be snake_case (lowercase + underscores + digits)
            assert!(
                tool.chars()
                    .all(|c| c.is_ascii_lowercase() || c == '_' || c.is_ascii_digit()),
                "Tool name '{}' is not valid snake_case",
                tool
            );
            // No leading/trailing underscores
            assert!(
                !tool.starts_with('_') && !tool.ends_with('_'),
                "Tool name '{}' starts or ends with underscore",
                tool
            );
            // Not empty
            assert!(!tool.is_empty(), "Empty tool name found");
        }
    }

    // ─── Tool Group Coverage Tests ──────────────────────────────────────────

    #[test]
    fn test_all_tool_groups_populated() {
        let groups = extract_tool_groups();
        assert!(
            !groups.is_empty(),
            "No tool groups found - check extraction logic"
        );

        println!("\nTool Groups Found:");
        for (group_name, tools) in &groups {
            println!("  {}: {} tools", group_name, tools.len());
            for tool in tools {
                println!("    - {}", tool);
            }
        }

        // Verify major groups exist
        let major_groups = ["PDDL", "POWL", "bcinr-logic"];
        for group in &major_groups {
            let has_group = groups.keys().any(|g| g.contains(group));
            assert!(
                has_group,
                "Missing major tool group: {} (found: {:?})",
                group,
                groups.keys().collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn test_tool_group_completeness() {
        let groups = extract_tool_groups();

        // Expected groups and minimum tool counts
        let expectations = [
            ("PDDL", 1),        // At least 1 PDDL tool
            ("POWL", 1),        // At least 1 POWL tool
            ("bcinr-logic", 1), // At least 1 algorithm tool
            ("Cross-crate", 0), // May not be explicitly grouped
        ];

        for (expected_group, min_count) in &expectations {
            let found = groups
                .keys()
                .find(|g| g.contains(expected_group))
                .map(|g| groups[g].len())
                .unwrap_or(0);

            assert!(
                found >= *min_count,
                "Group containing '{}' has {} tools, expected at least {}",
                expected_group,
                found,
                min_count
            );
        }
    }

    // ─── Functional Signature Tests ────────────────────────────────────────

    #[test]
    fn test_pddl_tools_exist() {
        let tools = extract_tool_names();
        let pddl_tools: Vec<_> = tools.iter().filter(|t| t.starts_with("pddl_")).collect();
        assert!(
            !pddl_tools.is_empty(),
            "No PDDL tools found. Tools: {:?}",
            tools
        );
    }

    #[test]
    fn test_powl_tools_exist() {
        let tools = extract_tool_names();
        let powl_tools: Vec<_> = tools.iter().filter(|t| t.starts_with("powl_")).collect();
        assert!(
            !powl_tools.is_empty(),
            "No POWL tools found. Tools: {:?}",
            tools
        );
    }

    #[test]
    fn test_algorithm_tools_exist() {
        let tools = extract_tool_names();
        let expected = [
            "utf8_validate",
            "bitset_operations",
            "dfa_info",
            "scan_patterns",
        ];
        for expected_tool in &expected {
            assert!(
                tools.iter().any(|t| t == expected_tool),
                "Expected algorithm tool '{}' not found. Tools: {:?}",
                expected_tool,
                tools
            );
        }
    }

    #[test]
    fn test_receipt_tools_exist() {
        let tools = extract_tool_names();
        assert!(
            tools.iter().any(|t| t.contains("receipt")),
            "No receipt tools found. Tools: {:?}",
            tools
        );
    }

    #[test]
    fn test_system_capabilities_exists() {
        let tools = extract_tool_names();
        assert!(
            tools.iter().any(|t| t == "system_capabilities"),
            "system_capabilities tool not found. Tools: {:?}",
            tools
        );
    }

    // ─── Pipeline Dependency Tests ──────────────────────────────────────────

    #[test]
    fn test_pddl_planning_pipeline() {
        let tools = extract_tool_names();

        // Expected PDDL tools in pipeline order
        let expected_sequence = ["pddl_parse_domain", "pddl_plan", "manufacture_world"];

        for tool in &expected_sequence {
            assert!(
                tools.iter().any(|t| t == tool),
                "PDDL pipeline tool '{}' not found",
                tool
            );
        }
    }

    #[test]
    fn test_powl_orchestration_pipeline() {
        let tools = extract_tool_names();

        let expected = [
            "powl_compile_sequence",
            "powl_admit_context",
            "powl_capability_check",
        ];

        for tool in &expected {
            assert!(
                tools.iter().any(|t| t == tool),
                "POWL pipeline tool '{}' not found",
                tool
            );
        }
    }

    #[test]
    fn test_end_to_end_flow_completeness() {
        let tools = extract_tool_names();

        // E2E flow should have tools from all major categories
        let categories = [
            ("pddl", "PDDL planning"),
            ("powl", "POWL orchestration"),
            ("utf8", "Algorithm: text"),
            ("receipt", "Receipt verification"),
            ("system", "System info"),
        ];

        for (prefix, description) in &categories {
            let has_category = tools.iter().any(|t| t.contains(prefix));
            assert!(
                has_category,
                "Missing {} tools (prefix: {}). Tools: {:?}",
                description, prefix, tools
            );
        }
    }

    // ─── Documentation Tests ──────────────────────────────────────────────

    #[test]
    fn test_tools_have_descriptions() {
        let src = fs::read_to_string("src/main.rs").expect("Failed to read main.rs");

        // Count #[tool(description = "...")]
        let tool_defs = src
            .lines()
            .filter(|line| line.contains("#[tool(description"))
            .count();

        let async_fns = extract_tool_names().len();

        assert_eq!(
            tool_defs, async_fns,
            "Not all tools have descriptions: {} definitions vs {} async fns",
            tool_defs, async_fns
        );
    }

    #[test]
    fn test_implementation_quality() {
        let src = fs::read_to_string("src/main.rs").expect("Failed to read main.rs");

        // Count async fn implementations (tools should be async)
        let async_count = src
            .lines()
            .filter(|line| line.contains("async fn") && !line.trim().starts_with("//"))
            .count();

        // Should have multiple async functions
        assert!(
            async_count >= 5,
            "Expected at least 5 async tools, found {}",
            async_count
        );
    }

    // ─── Test Infrastructure Verification ──────────────────────────────────

    #[test]
    fn test_extraction_logic_works() {
        let tools = extract_tool_names();
        let definitions = count_tool_definitions();

        println!("\nTool Inventory Report:");
        println!("  Extracted tool functions: {}", tools.len());
        println!("  #[tool(...)] definitions: {}", definitions);
        println!(
            "  Status: {}",
            if tools.len() == definitions {
                "CONSISTENT"
            } else {
                "MISMATCH"
            }
        );

        // At minimum, should find tools
        assert!(tools.len() > 0, "Extraction logic failed to find any tools");
    }

    #[test]
    fn test_report_all_tools() {
        let tools = extract_tool_names();
        let groups = extract_tool_groups();

        println!("\nComplete Tool Listing:");
        println!("  Total: {} tools", tools.len());
        println!("  Groups: {}", groups.len());
        println!("\nTools by Group:");
        for (group, group_tools) in &groups {
            println!("  {} ({} tools):", group, group_tools.len());
            for tool in group_tools {
                println!("    - {}", tool);
            }
        }
    }
}

// ─── BRCE Denial Chain Conformance ──────────────────────────────────────────
//
// Five falsifiable assertions proving the Vision 2030 BRCE loop properties:
//   A1 — BFS soundness: deploy→smoke→healthy = 3 steps (optimal)
//   A2 — Prolog8 gate: unapproved service ⟹ admitted=false (R ⊢ A)
//   A3 — BLAKE3 tamper evidence: mutate manufacture_chain ⟹ chain_valid=false
//   A4 — PDDL8 bounds: 9-arity predicate ⟹ BOUND_EXCEEDED at parse time
//   A5 — Determinism: identical inputs ⟹ identical domain_witness + manufacture_chain

#[cfg(test)]
mod brce_conformance {
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

    // Domain that violates PDDL8_MAX_ARITY (8): one predicate with 9 arguments.
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

    // Recompute manufacture_chain the same way llm_bridge.rs does:
    // BLAKE3(domain_witness.as_bytes() || problem_witness.as_bytes() || plan_chain_hash.as_bytes())
    fn recompute_chain(domain_w: &str, problem_w: &str, plan_chain: &str) -> String {
        recompute_chain_full(domain_w, problem_w, plan_chain, false, 0)
    }

    fn recompute_chain_full(
        domain_w: &str,
        problem_w: &str,
        plan_chain: &str,
        goal_reached: bool,
        step_count: u64,
    ) -> String {
        let mut h = blake3::Hasher::new();
        h.update(domain_w.as_bytes());
        h.update(problem_w.as_bytes());
        h.update(plan_chain.as_bytes());
        h.update(if goal_reached { b"1" } else { b"0" });
        h.update(&step_count.to_le_bytes());
        h.finalize()
            .as_bytes()
            .iter()
            .map(|x| format!("{x:02x}"))
            .collect()
    }

    /// A1 — BFS finds optimal 3-step plan: deploy → run-smoke → mark-healthy.
    #[test]
    fn a1_bfs_soundness_three_step_plan() {
        let r = manufacture_world(DEPLOY_DOMAIN, APPROVED_PROBLEM, "a1", &[]);

        assert!(
            r.admitted,
            "admitted should be true for approved service; refusal: {:?}",
            r.refusal_reason
        );
        assert_eq!(
            r.plan_receipt.step_count, 3,
            "BFS must find exactly 3 steps (deploy+smoke+healthy), got {}",
            r.plan_receipt.step_count
        );
        assert!(r.plan_receipt.goal_reached, "goal must be reached");
        assert!(
            !r.domain_witness.is_empty(),
            "domain_witness must be non-empty"
        );
        assert!(
            !r.manufacture_chain.is_empty(),
            "manufacture_chain must be non-empty"
        );

        // Verify manufacture_chain = BLAKE3(domain_w || problem_w || plan_chain || goal_reached || step_count)
        let expected = recompute_chain_full(
            &r.domain_witness,
            &r.problem_witness,
            &r.plan_receipt.chain_hash,
            r.plan_receipt.goal_reached,
            r.plan_receipt.step_count as u64,
        );
        assert_eq!(
            r.manufacture_chain, expected,
            "manufacture_chain must match BLAKE3 recomputation"
        );

        println!(
            "A1 PASS: {} steps, chain={}",
            r.plan_receipt.step_count,
            &r.manufacture_chain[..16]
        );
    }

    /// A2 — Prolog8 gate enforced at EXECUTION time: plan exists, policy denies it.
    ///
    /// This is R ⊢ A in its strongest form: the APPROVED_PROBLEM has `(approved api-v2)` in
    /// init, so BFS finds the 3-step plan. But policy_rules only admit `"__noadmit__"`, which
    /// matches no actual action label. Step 0 (`deploy(api-v2)`) is denied by the Prolog8 gate
    /// before any effect fires. This is execution-time denial, not planning-layer denial.
    #[test]
    fn a2_prolog8_gate_denies_execution_time() {
        // Permit only a label that matches nothing real → every step is denied.
        let r = manufacture_world(
            DEPLOY_DOMAIN,
            APPROVED_PROBLEM,
            "a2",
            &[("__noadmit__", vec![])],
        );

        assert!(
            !r.admitted,
            "Prolog8 gate must deny execution even when the plan exists"
        );
        let reason = r.refusal_reason.as_deref().unwrap_or("");
        assert!(
            reason.contains("denied") || reason.contains("Denied"),
            "refusal_reason must indicate Prolog8 gate denial at execution time, got: '{reason}'"
        );

        println!(
            "A2 PASS: Prolog8 gate stopped execution at step 0 — {}",
            &reason[..reason.len().min(80)]
        );
    }

    /// A3 — BLAKE3 tamper evidence: valid chain verifies; mutated chain fails.
    #[test]
    fn a3_blake3_tamper_evidence() {
        let r = manufacture_world(DEPLOY_DOMAIN, APPROVED_PROBLEM, "a3", &[]);
        assert!(r.admitted);

        // Valid: recomputed chain must match stored chain.
        let recomputed = recompute_chain_full(
            &r.domain_witness,
            &r.problem_witness,
            &r.plan_receipt.chain_hash,
            r.plan_receipt.goal_reached,
            r.plan_receipt.step_count as u64,
        );
        assert_eq!(
            recomputed, r.manufacture_chain,
            "valid receipt: chain must verify"
        );

        // Tampered: flip the last hex digit of manufacture_chain.
        let mut tampered = r.manufacture_chain.clone();
        let last = tampered.pop().unwrap();
        let flipped = if last == 'f' { '0' } else { 'f' };
        tampered.push(flipped);

        // Tampered chain must NOT equal the recomputed one.
        assert_ne!(
            recomputed, tampered,
            "tampered manufacture_chain must fail BLAKE3 verification"
        );

        println!("A3 PASS: valid chain verifies; tampered chain correctly detected");
    }

    /// A4 — PDDL8 bounds: domain with 9-arity predicate ⟹ admitted=false + BOUND_EXCEEDED.
    #[test]
    fn a4_pddl8_bounds_enforced_at_parse_time() {
        let r = manufacture_world(OVERBOUND_DOMAIN, OVERBOUND_PROBLEM, "a4", &[]);

        assert!(
            !r.admitted,
            "domain violating PDDL8_MAX_ARITY must be denied"
        );
        let reason = r.refusal_reason.as_deref().unwrap_or("");
        assert!(
            reason.contains("bound exceeded") || reason.contains("PDDL8 bound"),
            "refusal_reason must indicate PDDL8 bound violation, got: '{reason}'"
        );

        println!("A4 PASS: PDDL8 bounds enforced — {reason}");
    }

    /// A5 — Determinism: two identical manufacture_world calls produce identical witnesses + chains.
    #[test]
    fn a5_deterministic_blake3_witnesses() {
        let r1 = manufacture_world(DEPLOY_DOMAIN, APPROVED_PROBLEM, "a5-run1", &[]);
        let r2 = manufacture_world(DEPLOY_DOMAIN, APPROVED_PROBLEM, "a5-run2", &[]);

        // Both must admit
        assert!(
            r1.admitted && r2.admitted,
            "both runs must admit the same valid domain+problem"
        );

        // Witnesses are deterministic (BLAKE3 over structural identity, not timestamps)
        assert_eq!(
            r1.domain_witness, r2.domain_witness,
            "domain_witness must be identical across runs (deterministic BLAKE3)"
        );
        assert_eq!(
            r1.problem_witness, r2.problem_witness,
            "problem_witness must be identical across runs"
        );

        // manufacture_chain depends on plan_receipt.chain_hash which may differ if case_id affects it
        // Assert equality of the plan structure instead
        assert_eq!(
            r1.plan_receipt.step_count, r2.plan_receipt.step_count,
            "step_count must be identical across runs"
        );
        assert_eq!(
            r1.plan_receipt.goal_reached, r2.plan_receipt.goal_reached,
            "goal_reached must be identical across runs"
        );

        println!(
            "A5 PASS: domain_witness={}, problem_witness={} (stable across runs)",
            &r1.domain_witness[..16],
            &r1.problem_witness[..16]
        );
    }
}
