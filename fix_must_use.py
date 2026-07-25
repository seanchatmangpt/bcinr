import re
with open("crates/bcinr-logic/src/autonomic/auto_select_ocel_emission.rs", "r") as f:
    content = f.read()

content = content.replace("emit_ocel_trace(&mut s1, &t3);", "let _ = emit_ocel_trace(&mut s1, &t3);")
content = content.replace("emit_ocel_trace(&mut s1, &t4);", "let _ = emit_ocel_trace(&mut s1, &t4);")
content = content.replace("emit_ocel_trace(&mut s1, &t5);", "let _ = emit_ocel_trace(&mut s1, &t5);")

with open("crates/bcinr-logic/src/autonomic/auto_select_ocel_emission.rs", "w") as f:
    f.write(content)
