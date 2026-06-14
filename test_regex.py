import re
with open("./crates/bcinr-logic/src/algorithms/unique_branchless_u32.rs", "r") as f:
    content = f.read()

new_content = re.sub(
    r'if val != aux && val != 0 && aux != 0 \{\s*(prop_assert!\([^)]+\);)\s*\}',
    r'prop_assume!(val != aux && val != 0 && aux != 0);\n            \1',
    content
)

if new_content != content:
    print("Regex matched and replaced!")
else:
    print("Regex failed to match!")
