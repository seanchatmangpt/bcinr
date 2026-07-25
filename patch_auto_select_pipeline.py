import re

with open("crates/bcinr-powl/src/auto_select_pipeline.rs", "r") as f:
    content = f.read()

content = content.replace("use bcinr_logic::autonomic::auto_select::AutoSelectResult;\n", "")

# We want to replace the instantiation of auto_res and the call to powl_bridge_select
content = re.sub(
    r'let auto_res = AutoSelectResult \{\s*is_ok: canonical_res\.is_ok,\s*tool_id: canonical_res\.tool_id,\s*refusal_code: canonical_res\.refusal_code,\s*\};\s*let tape_mask = powl_bridge_select\(&auto_res\);',
    r'let tape_mask = powl_bridge_select(&canonical_res);',
    content
)

content = re.sub(
    r'PipelineIntegrationResult \{\s*is_ok: auto_res\.is_ok,\s*tape_mask,\s*refusal_code: auto_res\.refusal_code,\s*\}',
    r'PipelineIntegrationResult {\n        is_ok: canonical_res.is_ok,\n        tape_mask,\n        refusal_code: canonical_res.refusal_code,\n    }',
    content
)

with open("crates/bcinr-powl/src/auto_select_pipeline.rs", "w") as f:
    f.write(content)
