import re

with open("crates/bcinr-powl/src/mapek_loop.rs", "r") as f:
    content = f.read()

content = content.replace("let mut refusal_code;", "let refusal_code;")

with open("crates/bcinr-powl/src/mapek_loop.rs", "w") as f:
    f.write(content)
