import re

# Fix oracle_full_mapek_loop
with open("crates/bcinr-powl/src/full_mapek_loop.rs", "r") as f:
    code = f.read()

old_oracle_inner = """            let trace_res = log_execution_trace(trace_state, &input.trace);
            if trace_res.refusal_code == 0 {
                let safe_cursor = trace_state.cursor as usize % P;
                trace_state.frames[safe_cursor] = input.trace;
                trace_state.cursor += 1;
            }
            refusal_code |= trace_res.refusal_code;"""
new_oracle_inner = """            let trace_res = log_execution_trace(trace_state, &input.trace);
            refusal_code |= trace_res.refusal_code;"""
code = code.replace(old_oracle_inner, new_oracle_inner)

# Fix test_full_mapek_equivalence instruction_id
code = code.replace("input.trace.ts_ns = 100;", "input.trace.ts_ns = 100;\n        input.trace.instruction_id = 1;")

# Fix test_full_mapek_mutants instruction_id
code = code.replace("input.trace.ts_ns = 100;\n\n        let mut sub_ref", "input.trace.ts_ns = 100;\n        input.trace.instruction_id = 1;\n\n        let mut sub_ref")

# Wait, the double declaration `let mut t_ref = TraceBufferState::<16>::default();` etc.
# I had a regex that replaced `let mut (o[a-z0-9_]*) = OcelBufferState::<4>::default\(\);`
# It might have been run twice? Let's fix that.
code = re.sub(r'let mut t([a-z0-9_]*) = TraceBufferState::<16>::default\(\);\s*let mut t\1 = TraceBufferState::<4>::default\(\);', r'let mut t\1 = TraceBufferState::<4>::default();', code)

with open("crates/bcinr-powl/src/full_mapek_loop.rs", "w") as f:
    f.write(code)

with open("crates/bcinr-powl/src/mapek_loop.rs", "r") as f:
    code = f.read()
# Fix mapek_loop.rs tests just in case instruction_id is missing
code = code.replace("input.trace.ts_ns = 100;\n\n        let mut sub_ref", "input.trace.ts_ns = 100;\n        input.trace.instruction_id = 1;\n\n        let mut sub_ref")
code = code.replace("input.trace.ts_ns = 100;\n        let mut w_candidate", "input.trace.ts_ns = 100;\n        input.trace.instruction_id = 1;\n        let mut w_candidate")

with open("crates/bcinr-powl/src/mapek_loop.rs", "w") as f:
    f.write(code)

