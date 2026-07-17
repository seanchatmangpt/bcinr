import re

with open('crates/bcinr-cmca/tests/case_studies.rs', 'r') as f:
    content = f.read()

# Since we want the generated constants from C4 but also the admit_* calls, 
# wait, let me check the conflict in tests/case_studies.rs
