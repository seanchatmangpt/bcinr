import re

with open("crates/bcinr-powl/src/auto_select_pipeline.rs", "r") as f:
    content = f.read()

# Replace the specific block at line 151
content = re.sub(
    r'        let auto_res = AutoSelectResult \{\s*is_ok: canonical_res\.is_ok,\s*tool_id: canonical_res\.tool_id,\s*refusal_code: canonical_res\.refusal_code,\s*\};\s*let tape_mask = powl_bridge_select\(&auto_res\);',
    r'        let tape_mask = powl_bridge_select(&canonical_res);',
    content
)

with open("crates/bcinr-powl/src/auto_select_pipeline.rs", "w") as f:
    f.write(content)
