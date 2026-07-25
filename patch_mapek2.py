import re

with open("crates/bcinr-powl/src/mapek_loop.rs", "r") as f:
    content = f.read()

# Fix unused import
content = content.replace("integrate_auto_select_pipeline, PipelineIntegrationInput, PipelineIntegrationResult,", "integrate_auto_select_pipeline, PipelineIntegrationInput,")

# Fix unused assignment
content = content.replace("let mut refusal_code = 0;", "let mut refusal_code;")

with open("crates/bcinr-powl/src/mapek_loop.rs", "w") as f:
    f.write(content)
