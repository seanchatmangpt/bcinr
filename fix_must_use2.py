import re
with open("crates/bcinr-logic/src/autonomic/auto_select_ocel_emission.rs", "r") as f:
    content = f.read()

content = content.replace("let _ = let _ = emit_ocel_trace", "let _ = emit_ocel_trace")

with open("crates/bcinr-logic/src/autonomic/auto_select_ocel_emission.rs", "w") as f:
    f.write(content)
