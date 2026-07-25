import re
with open("crates/bcinr-powl/src/auto_select_pipeline.rs", "r") as f:
    content = f.read()

content = content.replace("AutoSelectResult,", "")

with open("crates/bcinr-powl/src/auto_select_pipeline.rs", "w") as f:
    f.write(content)

with open("crates/bcinr-cmca/tests/hostile_mutants.rs", "r") as f:
    content = f.read()

content = content.replace("ObservatoryFlag,", "")

with open("crates/bcinr-cmca/tests/hostile_mutants.rs", "w") as f:
    f.write(content)
