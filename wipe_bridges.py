import re
with open("crates/bcinr-powl/src/auto_select_pipeline.rs", "r") as f:
    content = f.read()

# Pattern for bridge_auto_res or bridge_res struct init
content = re.sub(r'let (?:bridge_)?(?:auto_)?res = bcinr_logic::autonomic::(?:auto_select::)?AutoSelectResult \{[^}]+\};', '', content)
content = re.sub(r'let tape_mask = powl_bridge_select\(&(?:bridge_)?(?:auto_)?res\);', r'let tape_mask = powl_bridge_select(&canonical_res);', content)

# Fix PipelineIntegrationResult
content = re.sub(
    r'PipelineIntegrationResult \{\s*is_ok: (?:bridge_)?(?:auto_)?res\.is_ok,\s*tape_mask,\s*refusal_code: (?:bridge_)?(?:auto_)?res\.refusal_code,\s*\}',
    r'PipelineIntegrationResult {\n        is_ok: canonical_res.is_ok,\n        tape_mask,\n        refusal_code: canonical_res.refusal_code,\n    }',
    content
)

# wait, what if canonical_res isn't named canonical_res in one of them?
# Let's replace select_optimal_candidate assignees to canonical_res everywhere
content = re.sub(r'let (?:auto_res|canonical_res) = select_optimal_candidate', r'let canonical_res = select_optimal_candidate', content)

with open("crates/bcinr-powl/src/auto_select_pipeline.rs", "w") as f:
    f.write(content)
