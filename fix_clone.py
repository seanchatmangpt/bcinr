import re
with open("crates/bcinr-powl-receipt/tests/hostile_mutants.rs", "r") as f:
    content = f.read()

content = content.replace("original.clone()", "original")

with open("crates/bcinr-powl-receipt/tests/hostile_mutants.rs", "w") as f:
    f.write(content)
