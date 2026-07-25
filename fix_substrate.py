import re
with open("crates/bcinr-logic/src/autonomic/autonomic_substrate.rs", "r") as f:
    content = f.read()

# Fix the duplicate derive
content = re.sub(
    r'#\[derive\([^\]]+\)\]\n\s*#\[derive\([^\]]+\)\]',
    r'#[derive(Clone, Copy, PartialEq, Eq, Debug)]',
    content
)
content = re.sub(
    r'#\[derive\(Clone, Copy, PartialEq, Eq, Debug, Debug, Clone, Copy, PartialEq, Eq\)\]',
    r'#[derive(Clone, Copy, PartialEq, Eq, Debug)]',
    content
)

with open("crates/bcinr-logic/src/autonomic/autonomic_substrate.rs", "w") as f:
    f.write(content)
