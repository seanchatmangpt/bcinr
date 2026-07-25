import re
with open("crates/bcinr-cmca/tests/hostile_mutants.rs", "r") as f:
    content = f.read()

content = re.sub(
    r'bcinr_cmca::observatory::(bcinr_cmca::observatory::)?,::',
    r'bcinr_cmca::observatory::ObservatoryFlag::',
    content
)

with open("crates/bcinr-cmca/tests/hostile_mutants.rs", "w") as f:
    f.write(content)
