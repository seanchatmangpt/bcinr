import re

with open("crates/bcinr-powl/src/auto_select_pipeline.rs", "r") as f:
    content = f.read()

replacement = """    let bridge_auto_res = bcinr_logic::autonomic::AutoSelectResult {
        is_ok: canonical_res.is_ok,
        tool_id: canonical_res.tool_id,
        refusal_code: canonical_res.refusal_code,
    };
    let tape_mask = powl_bridge_select(&bridge_auto_res);"""

# Replace `let tape_mask = powl_bridge_select(&canonical_res);` with the new block (accounting for optional blank lines and indentations)
content = re.sub(r"([ \t]+)let tape_mask = powl_bridge_select\(&canonical_res\);", 
                 lambda m: m.group(1) + replacement.replace("\n", "\n" + m.group(1)[4:] if len(m.group(1))>=4 else "\n"), 
                 content)

with open("crates/bcinr-powl/src/auto_select_pipeline.rs", "w") as f:
    f.write(content)

