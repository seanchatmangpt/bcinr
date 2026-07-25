import re

with open("crates/bcinr-powl/src/auto_select_pipeline.rs", "r") as f:
    content = f.read()

# Replace the first block
content = re.sub(
    r'let bridge_auto_res = bcinr_logic::autonomic::AutoSelectResult \{\s*is_ok: canonical_res\.is_ok,\s*tool_id: canonical_res\.tool_id,\s*refusal_code: canonical_res\.refusal_code,\s*\};\s*let tape_mask = powl_bridge_select\(&bridge_auto_res\);',
    r'let tape_mask = powl_bridge_select(&canonical_res);',
    content
)

# Replace the other blocks
content = re.sub(
    r'let bridge_auto_res = bcinr_logic::autonomic::canonical_mass::AutoSelectResult \{\s*is_ok: canonical_res\.is_ok,\s*tool_id: canonical_res\.tool_id,\s*refusal_code: canonical_res\.refusal_code,\s*\};\s*let tape_mask = powl_bridge_select\(&bridge_auto_res\);',
    r'let tape_mask = powl_bridge_select(&canonical_res);',
    content
)

with open("crates/bcinr-powl/src/auto_select_pipeline.rs", "w") as f:
    f.write(content)

