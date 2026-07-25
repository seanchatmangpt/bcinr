import re
with open("crates/bcinr-mcp/src/cache.rs", "r") as f:
    content = f.read()

# Just comment out the dead function entirely!
# It starts at `fn base<'a>() -> CapabilityCacheKey<'a> {`
# Let's just find and replace it
content = re.sub(r"fn base<'a>\(\) -> CapabilityCacheKey<'a> \{[^{}]*\}", "", content)

with open("crates/bcinr-mcp/src/cache.rs", "w") as f:
    f.write(content)
