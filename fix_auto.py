import os
import re

file_path = 'crates/bcinr-powl/src/auto_select_pipeline.rs'
with open(file_path, 'r') as f:
    content = f.read()

# Replace the block:
#         let bridge_auto_res = bcinr_logic::autonomic::AutoSelectResult {
#             is_ok: canonical_res.is_ok,
#             tool_id: canonical_res.tool_id,
#             refusal_code: canonical_res.refusal_code,
#         };
#         let tape_mask = powl_bridge_select(&bridge_auto_res);
# With:
#         let tape_mask = powl_bridge_select(&canonical_res);

pattern = re.compile(r'^[ \t]*let bridge_auto_res = bcinr_logic::autonomic::AutoSelectResult \{[ \t\n]*is_ok: canonical_res\.is_ok,[ \t\n]*tool_id: canonical_res\.tool_id,[ \t\n]*refusal_code: canonical_res\.refusal_code,[ \t\n]*\};[ \t\n]*let tape_mask = powl_bridge_select\(&bridge_auto_res\);', re.MULTILINE)

new_content = pattern.sub('    let tape_mask = powl_bridge_select(&canonical_res);', content)

with open(file_path, 'w') as f:
    f.write(new_content)
print("auto_select_pipeline.rs fixed")
