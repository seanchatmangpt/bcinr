import os
import re

DIR = "crates/bcinr-logic/src/algorithms/"

def replace_in_file(filename, old, new):
    path = os.path.join(DIR, filename)
    if not os.path.exists(path): return
    with open(path, 'r') as f:
        content = f.read()
    content = content.replace(old, new)
    with open(path, 'w') as f:
        f.write(content)

replace_in_file("funnel_shift_left_u64.rs", "64u64u64", "64u64")
replace_in_file("funnel_shift_right_u64.rs", "64u64u64", "64u64")
replace_in_file("fixed_point_log2.rs", "63u64u64", "63u64")

replace_in_file("metaphone_encode_branchless.rs", "c == b'A'", "c == 65")
replace_in_file("metaphone_encode_branchless.rs", "c == b'E'", "c == 69")
replace_in_file("metaphone_encode_branchless.rs", "c == b'I'", "c == 73")
replace_in_file("metaphone_encode_branchless.rs", "c == b'O'", "c == 79")
replace_in_file("metaphone_encode_branchless.rs", "c == b'U'", "c == 85")


