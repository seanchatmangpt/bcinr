import os

filepath = "crates/bcinr-powl/src/auto_select_final_integration.rs"
with open(filepath, "r") as f:
    content = f.read()

# 1. Imports
content = content.replace(
    "    auto_select_ocel_emission::OcelBufferState, autonomic_substrate::AutonomicSubstrate,",
    "    auto_select_ocel_emission::OcelBufferState, autonomic_substrate::AutonomicSubstrate,\n    auto_select_terminal_convergence::PersistentControlState,"
)

# 2. Signatures
content = content.replace(
    "    trace_state: &mut TraceBufferState<P>,\n) -> FullMapekResult {",
    "    trace_state: &mut TraceBufferState<P>,\n    terminal_state: &mut PersistentControlState,\n) -> FullMapekResult {"
)

content = content.replace(
    "    execute_full_mapek_loop(input, substrate, learning_weights, ocel_state, trace_state)\n}",
    "    execute_full_mapek_loop(input, substrate, learning_weights, ocel_state, trace_state, terminal_state)\n}"
)

content = content.replace(
    "    trace_state: &mut TraceBufferState<4>,\n) -> FullMapekResult {",
    "    trace_state: &mut TraceBufferState<4>,\n    terminal_state: &mut PersistentControlState,\n) -> FullMapekResult {"
)

content = content.replace(
    "    audit_execute_full_mapek_loop(input, substrate, learning_weights, ocel_state, trace_state)\n}",
    "    audit_execute_full_mapek_loop(input, substrate, learning_weights, ocel_state, trace_state, terminal_state)\n}"
)

# 3. Tests
content = content.replace(
    "        let mut trace_state = TraceBufferState::<4>::default();\n\n        let mut oracle_substrate = substrate.clone();",
    "        let mut trace_state = TraceBufferState::<4>::default();\n        let mut terminal_state = PersistentControlState::default();\n\n        let mut oracle_substrate = substrate.clone();"
)

content = content.replace(
    "            &mut trace_state,\n        );",
    "            &mut trace_state,\n            &mut terminal_state,\n        );"
)

with open(filepath, "w") as f:
    f.write(content)

print("Patched auto_select_final_integration.rs")
