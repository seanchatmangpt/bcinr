//! Integration tests for bcinr-mcp: Verify tool coverage without hardcoding
//!
//! These tests read tool definitions from main.rs and verify properties dynamically.
//! This ensures tests stay in sync with actual implementation.

#[cfg(test)]
mod tests {
    use serde_json::json;
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
                            groups
                                .get_mut(&current_group)
                                .map(|v| v.push(name));
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
            tools.len(), definitions,
            "Tool function count ({}) != #[tool(...)] count ({})",
            tools.len(), definitions
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
            tools.len(), unique_count,
            "Found duplicate tool names: {} unique out of {} total",
            unique_count, tools.len()
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
            ("PDDL", 1),                    // At least 1 PDDL tool
            ("POWL", 1),                    // At least 1 POWL tool
            ("bcinr-logic", 1),             // At least 1 algorithm tool
            ("Cross-crate", 0),             // May not be explicitly grouped
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
                expected_group, found, min_count
            );
        }
    }

    // ─── Functional Signature Tests ────────────────────────────────────────

    #[test]
    fn test_pddl_tools_exist() {
        let tools = extract_tool_names();
        let pddl_tools: Vec<_> = tools
            .iter()
            .filter(|t| t.starts_with("pddl_"))
            .collect();
        assert!(
            !pddl_tools.is_empty(),
            "No PDDL tools found. Tools: {:?}",
            tools
        );
    }

    #[test]
    fn test_powl_tools_exist() {
        let tools = extract_tool_names();
        let powl_tools: Vec<_> = tools
            .iter()
            .filter(|t| t.starts_with("powl_"))
            .collect();
        assert!(
            !powl_tools.is_empty(),
            "No POWL tools found. Tools: {:?}",
            tools
        );
    }

    #[test]
    fn test_algorithm_tools_exist() {
        let tools = extract_tool_names();
        let expected = ["utf8_validate", "bitset_operations", "dfa_info", "scan_patterns"];
        for expected_tool in &expected {
            assert!(
                tools.iter().any(|t| t == expected_tool),
                "Expected algorithm tool '{}' not found. Tools: {:?}",
                expected_tool, tools
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
        let expected_sequence = [
            "pddl_parse_domain",
            "pddl_plan",
            "manufacture_world",
        ];

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

        let expected = ["powl_compile_sequence", "powl_admit_context", "powl_capability_check"];

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

        println!("\n📊 Tool Inventory Report:");
        println!("  Extracted tool functions: {}", tools.len());
        println!("  #[tool(...)] definitions: {}", definitions);
        println!("  Status: {}", if tools.len() == definitions { "✓ CONSISTENT" } else { "⚠ MISMATCH" });

        // At minimum, should find tools
        assert!(tools.len() > 0, "Extraction logic failed to find any tools");
    }

    #[test]
    fn test_report_all_tools() {
        let tools = extract_tool_names();
        let groups = extract_tool_groups();

        println!("\n📋 Complete Tool Listing:");
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
