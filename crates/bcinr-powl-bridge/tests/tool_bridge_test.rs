//! Integration tests for bcinr-powl-bridge tool-call registry.

use bcinr_powl_bridge::{ToolCallRegistry, ToolCallSpec, dispatch_tool_call};

#[test]
fn test_registry_with_pick_up_mapping() {
    let mut registry = ToolCallRegistry::new();
    registry.register(
        "pick-up",
        ToolCallSpec::new("test_tool"),
    );

    // Verify the mapping exists
    let spec = registry.get("pick-up").expect("pick-up should be registered");
    assert_eq!(spec.tool_name, "test_tool");

    // Dispatch (mock mode — no real HTTP)
    let result = registry.dispatch(spec, "pick-up", &["block-a"]).unwrap();

    // chain_contribution must be a valid 32-byte value
    assert_eq!(result.chain_contribution.len(), 32);
    // Must not be all zeros (BLAKE3 of any input is non-trivial)
    assert_ne!(result.chain_contribution, [0u8; 32]);

    println!("chain_contribution (hex): {}", hex(&result.chain_contribution));
    println!("output: {}", result.output);
}

#[test]
fn test_dispatch_tool_call_standalone() {
    let spec = ToolCallSpec::new("test_tool");
    let result = dispatch_tool_call(&spec, &["arg1", "arg2"]).unwrap();
    assert_eq!(result.chain_contribution.len(), 32);
    assert!(result.output.is_object());
}

#[test]
fn test_multiple_registrations() {
    let mut registry = ToolCallRegistry::new();
    registry.register("pick-up", ToolCallSpec::new("grasp_tool"));
    registry.register("put-down", ToolCallSpec::new("release_tool"));
    registry.register("stack", ToolCallSpec::new("stack_tool"));

    assert!(registry.get("pick-up").is_some());
    assert!(registry.get("put-down").is_some());
    assert!(registry.get("stack").is_some());
    assert!(registry.get("unstack").is_none());
}

#[test]
fn test_chain_contributions_differ_per_step() {
    let spec = ToolCallSpec::new("test_tool");
    let r1 = dispatch_tool_call(&spec, &["a"]).unwrap();
    let r2 = dispatch_tool_call(&spec, &["b"]).unwrap();
    // Different args → different chain contributions
    assert_ne!(r1.chain_contribution, r2.chain_contribution);
}

#[test]
fn test_live_dispatcher_integration() {
    // Simulate what bcinr-mcp would do: provide a closure that returns JSON.
    let mut registry = ToolCallRegistry::with_dispatcher(Box::new(|tool_name, params| {
        // In real usage this would be an HTTP call to the MCP server.
        // Here we simulate a successful response.
        let response = serde_json::json!({
            "ok": true,
            "tool": tool_name,
            "echo": params,
        });
        Ok(response.to_string())
    }));

    registry.register("pick-up", ToolCallSpec::new("test_tool"));
    let spec = registry.get("pick-up").unwrap().clone();
    let result = registry.dispatch(&spec, "pick-up(a)", &["a"]).unwrap();

    assert_eq!(result.chain_contribution.len(), 32);
    assert_eq!(result.output["ok"], true);
    assert_eq!(result.output["tool"], "test_tool");
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
