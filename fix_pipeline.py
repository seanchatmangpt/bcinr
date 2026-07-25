import re
with open("crates/bcinr-powl/src/auto_select_pipeline.rs", "r") as f:
    content = f.read()

content = re.sub(
    r'(#\[repr\(C\)\]\n)?pub struct PipelineIntegrationResult \{',
    r'#[repr(C)]\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub struct PipelineIntegrationResult {',
    content
)

with open("crates/bcinr-powl/src/auto_select_pipeline.rs", "w") as f:
    f.write(content)
