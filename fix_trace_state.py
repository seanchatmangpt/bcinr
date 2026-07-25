import os
import re

def fix_file(filepath):
    if not os.path.exists(filepath): return
    with open(filepath, 'r') as f:
        code = f.read()

    # In full_mapek_loop.rs
    code = code.replace("execute_full_mapek_loop(&m_input, substrate, learning_weights, ocel_state)",
                        "execute_full_mapek_loop(&m_input, substrate, learning_weights, ocel_state, trace_state)")
    
    # In auto_select_final_integration.rs
    code = re.sub(r'execute_final_integration\(([^,]+),([^,]+),([^,]+),([^,)]+)\)',
                  r'execute_final_integration(\1,\2,\3,\4, trace_state)', code)
                  
    code = re.sub(r'final_integration_reference\(([^,]+),([^,]+),([^,]+),([^,)]+)\)',
                  r'final_integration_reference(\1,\2,\3,\4, trace_state)', code)

    code = re.sub(r'mutant_final_integration_1\(([^,]+),([^,]+),([^,]+),([^,)]+)\)',
                  r'mutant_final_integration_1(\1,\2,\3,\4, trace_state)', code)
                  
    code = re.sub(r'mutant_final_integration_2\(([^,]+),([^,]+),([^,]+),([^,)]+)\)',
                  r'mutant_final_integration_2(\1,\2,\3,\4, trace_state)', code)

    code = re.sub(r'mutant_final_integration_3\(([^,]+),([^,]+),([^,]+),([^,)]+)\)',
                  r'mutant_final_integration_3(\1,\2,\3,\4, trace_state)', code)
                  
    # Test instantiations in auto_select_final_integration.rs
    # They will have trace_state as undefined if we don't declare it
    if "auto_select_final_integration.rs" in filepath:
        # replace `let mut oX = OcelBufferState::<4>::default();` with it + trace_state
        code = re.sub(r'(let mut (o[a-z0-9_]*) = OcelBufferState::<4>::default\(\);)',
                      r'\1\n        let mut t\2 = TraceBufferState::<4>::default();', code)
        
        # Then replace the calls to pass the newly created tX
        # Wait, the regex above added trace_state literal instead of &mut tX
        # Let's fix that
        # In test, execute_final_integration(&input, &mut sub1, &mut w1, &mut o1, trace_state) -> &mut to1
        
        # Actually it's easier to just do it manually for the test section.
        code = code.replace(", trace_state);", ", &mut t1);") # Will fix later if it replaces wrong ones. Let's be precise.
        pass

    with open(filepath, 'w') as f:
        f.write(code)

fix_file("crates/bcinr-powl/src/full_mapek_loop.rs")

with open("crates/bcinr-powl/src/auto_select_final_integration.rs", "r") as f:
    code = f.read()

code = code.replace("execute_final_integration(input, substrate, learning_weights, ocel_state)", "execute_final_integration(input, substrate, learning_weights, ocel_state, trace_state)")
code = code.replace("execute_final_integration(&input, &mut sub1, &mut w1, &mut o1)", "execute_final_integration(&input, &mut sub1, &mut w1, &mut o1, &mut t1)")
code = code.replace("final_integration_reference(&input, &mut sub2, &mut w2, &mut o2)", "final_integration_reference(&input, &mut sub2, &mut w2, &mut o2, &mut t2)")
code = code.replace("final_integration_reference(&input, &mut sub_ref, &mut w_ref, &mut o_ref)", "final_integration_reference(&input, &mut sub_ref, &mut w_ref, &mut o_ref, &mut t_ref)")
code = code.replace("mutant_final_integration_1(&input, &mut sub1, &mut w1, &mut o1)", "mutant_final_integration_1(&input, &mut sub1, &mut w1, &mut o1, &mut t1)")
code = code.replace("mutant_final_integration_2(&input, &mut sub2, &mut w2, &mut o2)", "mutant_final_integration_2(&input, &mut sub2, &mut w2, &mut o2, &mut t2)")
code = code.replace("mutant_final_integration_3(&input, &mut sub3, &mut w3, &mut o3)", "mutant_final_integration_3(&input, &mut sub3, &mut w3, &mut o3, &mut t3)")

code = code.replace("let mut o1 = OcelBufferState::<4>::default();", "let mut o1 = OcelBufferState::<4>::default();\\n        let mut t1 = TraceBufferState::<4>::default();")
code = code.replace("let mut o2 = OcelBufferState::<4>::default();", "let mut o2 = OcelBufferState::<4>::default();\\n        let mut t2 = TraceBufferState::<4>::default();")
code = code.replace("let mut o3 = OcelBufferState::<4>::default();", "let mut o3 = OcelBufferState::<4>::default();\\n        let mut t3 = TraceBufferState::<4>::default();")
code = code.replace("let mut o_ref = OcelBufferState::<4>::default();", "let mut o_ref = OcelBufferState::<4>::default();\\n        let mut t_ref = TraceBufferState::<4>::default();")

with open("crates/bcinr-powl/src/auto_select_final_integration.rs", "w") as f:
    f.write(code.replace("\\n", "\n"))

