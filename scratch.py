import re

with open("crates/bcinr-powl/src/auto_select_final_integration.rs", "r") as f:
    code = f.read()

code = code.replace(
    "use crate::full_mapek_loop::{",
    "use bcinr_logic::autonomic::auto_select_trace_logging::TraceBufferState;\nuse crate::full_mapek_loop::{"
)

# 1. auto_select_final_integration signature
code = code.replace(
    "ocel_state: &mut OcelBufferState<O>,",
    "ocel_state: &mut OcelBufferState<O>,\n    trace_state: &mut TraceBufferState<P>,"
)
code = code.replace(
    "const O: usize,",
    "const O: usize,\n    const P: usize,"
)
code = code.replace(
    "execute_full_mapek_loop(input, substrate, learning_weights, ocel_state)",
    "execute_full_mapek_loop(input, substrate, learning_weights, ocel_state, trace_state)"
)

# 2. audit wrapper
code = code.replace(
    "ocel_state: &mut OcelBufferState<4>,",
    "ocel_state: &mut OcelBufferState<4>,\n    trace_state: &mut TraceBufferState<4>,"
)
code = code.replace(
    "audit_execute_full_mapek_loop(input, substrate, learning_weights, ocel_state)",
    "audit_execute_full_mapek_loop(input, substrate, learning_weights, ocel_state, trace_state)"
)


with open("crates/bcinr-powl/src/auto_select_final_integration.rs", "w") as f:
    f.write(code)

