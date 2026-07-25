import os, re

# Fix full_mapek_loop.rs
with open("crates/bcinr-powl/src/full_mapek_loop.rs", "r") as f:
    text = f.read()

text = text.replace("_terminal_state: &mut PersistentControlState", "terminal_state: &mut PersistentControlState")

# Fix missing terminal_state definition in full_mapek_loop test
text = text.replace(
"""        let res1 = execute_full_mapek_loop(&input, &mut sub1, &mut w1, &mut o1, &mut t1, &mut terminal_state);
        let res2 = oracle_full_mapek_loop(&input, &mut sub2, &mut w2, &mut o2, &mut t2, &mut oracle_terminal_state);""",
"""        let mut terminal_state = PersistentControlState::default();
        let mut oracle_terminal_state = PersistentControlState::default();
        let res1 = execute_full_mapek_loop(&input, &mut sub1, &mut w1, &mut o1, &mut t1, &mut terminal_state);
        let res2 = oracle_full_mapek_loop(&input, &mut sub2, &mut w2, &mut o2, &mut t2, &mut oracle_terminal_state);"""
)

with open("crates/bcinr-powl/src/full_mapek_loop.rs", "w") as f:
    f.write(text)

# Fix auto_select_final_integration.rs
with open("crates/bcinr-powl/src/auto_select_final_integration.rs", "r") as f:
    text = f.read()

text = text.replace(
"""        execute_full_mapek_loop(input, substrate, learning_weights, ocel_state, trace_state)""",
"""        execute_full_mapek_loop(input, substrate, learning_weights, ocel_state, trace_state, terminal_state)"""
)

# Test fixes
text = text.replace(
"""    fn mutant_final_bypassed_policy<
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
"""    fn mutant_final_bypassed_policy<
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
    ) -> FullMapekResult {"""
)
text = text.replace(
"""    fn mutant_final_state_drift<
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
"""    fn mutant_final_state_drift<
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
    ) -> FullMapekResult {"""
)

# inside tests
text = re.sub(
    r"(execute_final_integration\([^,]+, [^,]+, [^,]+, [^,]+, trace_state, &mut terminal_state)\)",
    r"execute_final_integration(input, substrate, learning_weights, ocel_state, trace_state, terminal_state)",
    text
)
# the previous python script changed `execute_final_integration(&input, &mut sub1, &mut w1, &mut o1, &mut _t1)` to `... &mut terminal_state.clone()`
text = text.replace("&mut terminal_state.clone()", "&mut terminal_state")

with open("crates/bcinr-powl/src/auto_select_final_integration.rs", "w") as f:
    f.write(text)

