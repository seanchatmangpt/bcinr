import re

with open("crates/bcinr-powl/src/auto_select_pipeline.rs", "r") as f:
    content = f.read()

# Fix ToolCandidate in oracle
replacement_oracle = """            let c = proj.candidate;
            auto_input.candidates[i] = bcinr_logic::autonomic::canonical_mass::ToolCandidate {
                tool_id: c.tool_id,
                semantic_fit: c.semantic_fit,
                evidence_fit: c.evidence_fit,
                authority_fit: c.authority_fit,
                timing_fit: c.timing_fit,
                downstream_fit: c.downstream_fit,
                reliability: c.reliability,
                cost_fit: c.cost_fit,
                mass: c.mass,
            };"""
content = content.replace("auto_input.candidates[i] = proj.candidate;", replacement_oracle)

# Fix AutoSelectResult everywhere before calling powl_bridge_select
def replace_bridge(match):
    return """
    let bridge_auto_res = bcinr_logic::autonomic::auto_select::AutoSelectResult {
        is_ok: auto_res.is_ok,
        tool_id: auto_res.tool_id,
        refusal_code: auto_res.refusal_code,
    };
    let tape_mask = powl_bridge_select(&bridge_auto_res);
    """

content = re.sub(r'let tape_mask = powl_bridge_select\(&auto_res\);', replace_bridge, content)

with open("crates/bcinr-powl/src/auto_select_pipeline.rs", "w") as f:
    f.write(content)
