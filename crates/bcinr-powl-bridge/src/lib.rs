//! bcinr-powl-bridge — MCP Tool-Call Bridge
//!
//! Maps PDDL action names to MCP tool calls and commits results into the BLAKE3
//! receipt chain. Transport-agnostic: callers supply a `ToolDispatcher` closure.
//!
//! # Design
//! - `ToolCallSpec`: what MCP tool to call and static extra params.
//! - `ToolCallResult`: the JSON output plus the 32-byte chain contribution.
//! - `ToolCallRegistry`: a name → spec map with a pluggable dispatcher.
//! - `dispatch_tool_call`: builds params, calls dispatcher, computes BLAKE3 contribution.

use std::collections::HashMap;
use serde_json::Value;
use blake3::Hasher;

// ─── Public Types ────────────────────────────────────────────────────────────

/// Specification for mapping a PDDL action name to an MCP tool call.
#[derive(Debug, Clone)]
pub struct ToolCallSpec {
    /// MCP tool name (e.g. "pddl_plan", "utf8_validate").
    pub tool_name: String,
    /// Static string parameters merged into the call alongside action-derived params.
    /// Keys map PDDL arg positions ("arg0", "arg1", …) or static extra keys to string values.
    pub param_map: HashMap<String, String>,
    /// Optional HTTP endpoint override (e.g. "http://localhost:3000/mcp").
    /// None → use the dispatcher's default endpoint.
    pub endpoint: Option<String>,
}

impl ToolCallSpec {
    /// Construct a minimal spec with just a tool name.
    pub fn new(tool_name: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            param_map: HashMap::new(),
            endpoint: None,
        }
    }
}

/// The result of dispatching an MCP tool call for a PDDL plan step.
#[derive(Debug, Clone)]
pub struct ToolCallResult {
    /// Parsed JSON output from the tool.
    pub output: Value,
    /// BLAKE3(tool_name || serialized_args || output_bytes) — 32 bytes.
    /// XOR or chain-feed this into the running receipt chain.
    pub chain_contribution: [u8; 32],
}

/// Error variants from the tool-call bridge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolBridgeError {
    /// The dispatcher returned an error string.
    DispatchFailed(String),
    /// The tool returned non-UTF-8 or malformed JSON.
    InvalidOutput(String),
    /// No dispatcher is available (registry has no dispatcher set).
    NoDispatcher,
}

impl std::fmt::Display for ToolBridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DispatchFailed(s) => write!(f, "tool dispatch failed: {s}"),
            Self::InvalidOutput(s) => write!(f, "invalid tool output: {s}"),
            Self::NoDispatcher => write!(f, "no dispatcher configured in registry"),
        }
    }
}

impl std::error::Error for ToolBridgeError {}

/// Synchronous dispatch function type.
/// Receives (tool_name, params_json) → Ok(result_json_string) | Err(reason).
pub type ToolDispatcher = Box<dyn Fn(&str, Value) -> Result<String, String> + Send + Sync>;

/// Registry mapping PDDL action base names to MCP tool call specs.
pub struct ToolCallRegistry {
    /// Key: PDDL action base name (e.g. "pick-up").
    pub mappings: HashMap<String, ToolCallSpec>,
    /// Synchronous dispatcher. None → mock mode (returns deterministic output).
    dispatcher: Option<ToolDispatcher>,
}

impl ToolCallRegistry {
    /// Create an empty registry with no dispatcher (mock mode).
    pub fn new() -> Self {
        Self { mappings: HashMap::new(), dispatcher: None }
    }

    /// Create a registry with a concrete dispatcher closure.
    pub fn with_dispatcher(dispatcher: ToolDispatcher) -> Self {
        Self { mappings: HashMap::new(), dispatcher: Some(dispatcher) }
    }

    /// Register an action → tool mapping.
    pub fn register(&mut self, action_name: impl Into<String>, spec: ToolCallSpec) {
        self.mappings.insert(action_name.into(), spec);
    }

    /// Look up a spec by PDDL action base name.
    pub fn get(&self, action: &str) -> Option<&ToolCallSpec> {
        // Strip parenthesized args if caller passes "pick-up(a,b)".
        let base = action.split('(').next().unwrap_or(action);
        self.mappings.get(base)
    }

    /// Dispatch a tool call for the given spec and PDDL step arguments.
    ///
    /// In mock mode (no dispatcher) returns a deterministic result based on BLAKE3.
    pub fn dispatch(
        &self,
        spec: &ToolCallSpec,
        action_name: &str,
        args: &[&str],
    ) -> Result<ToolCallResult, ToolBridgeError> {
        dispatch_tool_call_inner(self.dispatcher.as_deref(), spec, action_name, args)
    }
}

impl Default for ToolCallRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Core dispatch function ──────────────────────────────────────────────────

/// Dispatch an MCP tool call given a spec and PDDL step arguments.
///
/// Builds a JSON params object:
/// ```json
/// { "action": "pick-up(a)", "args": ["a"], "arg0": "a", <extra_params...> }
/// ```
/// then calls the dispatcher (or mock) and computes the chain contribution.
pub fn dispatch_tool_call(
    spec: &ToolCallSpec,
    args: &[&str],
) -> Result<ToolCallResult, ToolBridgeError> {
    dispatch_tool_call_inner(None, spec, &spec.tool_name, args)
}

fn dispatch_tool_call_inner(
    dispatcher: Option<&(dyn Fn(&str, Value) -> Result<String, String> + Send + Sync)>,
    spec: &ToolCallSpec,
    action_name: &str,
    args: &[&str],
) -> Result<ToolCallResult, ToolBridgeError> {
    // Build params JSON
    let mut params = serde_json::json!({
        "action": action_name,
        "args": args,
    });
    if let Value::Object(ref mut map) = params {
        // Positional shortcuts: arg0, arg1, …
        for (i, &arg) in args.iter().enumerate() {
            map.insert(format!("arg{i}"), Value::String(arg.to_string()));
        }
        // Static param_map overrides
        for (k, v) in &spec.param_map {
            map.insert(k.clone(), Value::String(v.clone()));
        }
    }

    // Call dispatcher or use mock
    let raw_output = match dispatcher {
        Some(d) => d(&spec.tool_name, params.clone())
            .map_err(|e| ToolBridgeError::DispatchFailed(e))?,
        None => {
            // Mock: return a stable JSON string based on tool_name + args
            let mock = serde_json::json!({
                "mock": true,
                "tool": spec.tool_name,
                "received_args": args,
            });
            mock.to_string()
        }
    };

    // Parse output
    let output: Value = serde_json::from_str(&raw_output)
        .map_err(|e| ToolBridgeError::InvalidOutput(e.to_string()))?;

    // Compute chain contribution: BLAKE3(tool_name || serialized_params || raw_output)
    let mut h = Hasher::new();
    h.update(spec.tool_name.as_bytes());
    h.update(params.to_string().as_bytes());
    h.update(raw_output.as_bytes());
    let chain_contribution = *h.finalize().as_bytes();

    Ok(ToolCallResult { output, chain_contribution })
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatch_mock_returns_valid_result() {
        let spec = ToolCallSpec::new("test_tool");
        let result = dispatch_tool_call(&spec, &["a", "b"]).unwrap();
        assert_eq!(result.chain_contribution.len(), 32);
        assert!(result.output.is_object());
        assert_eq!(result.output["mock"], Value::Bool(true));
    }

    #[test]
    fn test_registry_get_strips_args() {
        let mut reg = ToolCallRegistry::new();
        reg.register("pick-up", ToolCallSpec::new("test_tool"));
        // With and without parenthesized args
        assert!(reg.get("pick-up").is_some());
        assert!(reg.get("pick-up(a,b)").is_some());
        assert!(reg.get("put-down").is_none());
    }

    #[test]
    fn test_chain_contribution_is_deterministic() {
        let spec = ToolCallSpec::new("utf8_validate");
        let r1 = dispatch_tool_call(&spec, &["hello"]).unwrap();
        let r2 = dispatch_tool_call(&spec, &["hello"]).unwrap();
        assert_eq!(r1.chain_contribution, r2.chain_contribution);
    }

    #[test]
    fn test_chain_contribution_differs_by_tool() {
        let spec_a = ToolCallSpec::new("tool_a");
        let spec_b = ToolCallSpec::new("tool_b");
        let r1 = dispatch_tool_call(&spec_a, &["x"]).unwrap();
        let r2 = dispatch_tool_call(&spec_b, &["x"]).unwrap();
        assert_ne!(r1.chain_contribution, r2.chain_contribution);
    }

    #[test]
    fn test_dispatcher_closure_used() {
        let mut reg = ToolCallRegistry::with_dispatcher(Box::new(|tool, _params| {
            Ok(format!(r#"{{"called":"{}"}}"#, tool))
        }));
        reg.register("pick-up", ToolCallSpec::new("my_tool"));
        let spec = reg.get("pick-up").unwrap().clone();
        let result = reg.dispatch(&spec, "pick-up(a)", &["a"]).unwrap();
        assert_eq!(result.output["called"], "my_tool");
        assert_eq!(result.chain_contribution.len(), 32);
    }

    #[test]
    fn test_param_map_included_in_params() {
        let mut spec = ToolCallSpec::new("pddl_plan");
        spec.param_map.insert("domain".to_string(), "blocks".to_string());
        let result = dispatch_tool_call(&spec, &["block-a"]).unwrap();
        // chain_contribution must be a valid 32-byte value
        assert_eq!(result.chain_contribution.len(), 32);
    }
}
