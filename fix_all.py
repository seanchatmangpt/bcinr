import re

with open("crates/bcinr-powl/src/auto_select_final_integration.rs", "r") as f:
    text = f.read()

# Fix final_integration_reference signature
text = text.replace(
"""    fn final_integration_reference<
        K: Copy + Default + PartialEq,
        V: Copy + Default,
        const N: usize,
        const O: usize,
        const P: usize,
    >(
        input: &FullMapekInput,
        substrate: &mut AutonomicSubstrate<K, V, N>,
        learning_weights: &mut LearningWeights,
        ocel_state: &mut OcelBufferState<O>,
        trace_state: &mut TraceBufferState<P>,
    ) -> FullMapekResult {""",
"""    fn final_integration_reference<
        K: Copy + Default + PartialEq,
        V: Copy + Default,
        const N: usize,
        const O: usize,
        const P: usize,
    >(
        input: &FullMapekInput,
        substrate: &mut AutonomicSubstrate<K, V, N>,
        learning_weights: &mut LearningWeights,
        ocel_state: &mut OcelBufferState<O>,
        trace_state: &mut TraceBufferState<P>,
        terminal_state: &mut PersistentControlState,
    ) -> FullMapekResult {""")

# Fix final_integration_reference calls in test_counterfactual_mutants
text = text.replace(
    "final_integration_reference(&input, &mut sub_ref, &mut w_ref, &mut o_ref, &mut _t_ref);",
    "final_integration_reference(&input, &mut sub_ref, &mut w_ref, &mut o_ref, &mut _t_ref, &mut terminal_state);"
)

with open("crates/bcinr-powl/src/auto_select_final_integration.rs", "w") as f:
    f.write(text)

with open("crates/bcinr-powl/src/full_mapek_loop.rs", "r") as f:
    text = f.read()

text = text.replace("let mut intermediate_refusal", "let intermediate_refusal")
text = text.replace("let m_term_commit", "let _m_term_commit")
text = text.replace("let mut terminal_state = PersistentControlState::default();", "let terminal_state = PersistentControlState::default();")

with open("crates/bcinr-powl/src/full_mapek_loop.rs", "w") as f:
    f.write(text)
