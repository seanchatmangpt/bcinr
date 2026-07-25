import re

with open("crates/bcinr-powl/src/auto_select_final_integration.rs", "r") as f:
    text = f.read()

# Add terminal_state to mutant signatures
text = re.sub(
    r"(fn mutant_final_integration_\d<[^>]+>\([^)]+trace_state: &mut TraceBufferState<P>,\n    ) -> FullMapekResult \{",
    r"\1terminal_state: &mut PersistentControlState,\n    ) -> FullMapekResult {",
    text
)

# Fix test_equivalence
text = text.replace(
    "let mut w1 = LearningWeights::default();",
    "let mut terminal_state = PersistentControlState::default();\n        let mut w1 = LearningWeights::default();"
)

with open("crates/bcinr-powl/src/auto_select_final_integration.rs", "w") as f:
    f.write(text)
