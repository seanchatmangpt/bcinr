import re

with open("crates/bcinr-powl/src/auto_select_pipeline.rs", "r") as f:
    content = f.read()

# First, remove AutoSelectRefusal from imports
content = content.replace("select_optimal_candidate, AutoSelectInput8, AutoSelectRefusal", "select_optimal_candidate, AutoSelectInput8")

# Let's revert my bridge_auto_res hack
content = re.sub(r'let bridge_auto_res = .*?let tape_mask = powl_bridge_select\(&bridge_auto_res\);', 'let tape_mask = powl_bridge_select(&auto_res);', content, flags=re.DOTALL)

with open("crates/bcinr-powl/src/auto_select_pipeline.rs", "w") as f:
    f.write(content)
