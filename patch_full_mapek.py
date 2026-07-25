import os

filepath = "crates/bcinr-powl/src/full_mapek_loop.rs"
with open(filepath, "r") as f:
    content = f.read()

# 1. Imports
content = content.replace(
    "    auto_select_substrate_convergence::substrate_convergence,\n",
    "    auto_select_substrate_convergence::substrate_convergence,\n    auto_select_terminal_convergence::{terminal_convergence, TerminalConvergenceInput, PersistentControlState, RefusalAggregationState},\n"
)

# 2. Input Struct
content = content.replace(
    "    pub trace: OcelCausalFrame,\n}",
    "    pub trace: OcelCausalFrame,\n    pub terminal_input: TerminalConvergenceInput,\n}"
)
content = content.replace(
    "            trace: OcelCausalFrame::default(),\n        }",
    "            trace: OcelCausalFrame::default(),\n            terminal_input: TerminalConvergenceInput { m_tape: 0, r_aggr: RefusalAggregationState { critical_refusal: 0 }, expected_epoch: 0 },\n        }"
)

# 3. execute_full_mapek_loop signature
content = content.replace(
    "    trace_state: &mut TraceBufferState<P>,\n) -> FullMapekResult {",
    "    trace_state: &mut TraceBufferState<P>,\n    terminal_state: &mut PersistentControlState,\n) -> FullMapekResult {"
)

# 4. In execute_full_mapek_loop, Refusal Aggregation is already at the bottom.
# We need to compute r_aggr, then call terminal_convergence.
old_refusal = """    let refusal_input = crate::auto_select_refusal_aggregation::RefusalAggregationInput {
        r_base: base_refusal,
        r_adapt: adapt_res.refusal_code,
        r_dispatch: dispatch_res.refusal_code,
        r_conv: convergence_res.refusal_code,
        r_receipt: ingest_res.refusal_code,
        r_ocel: ocel_res.refusal_code,
        r_trace: trace_res.refusal_code,
        r_epoch: epoch_res.refusal_code,
        m_update: m_update as u8,
    };
    let refusal_code = crate::auto_select_refusal_aggregation::aggregate_refusals(&refusal_input);

    FullMapekResult {"""

new_refusal = """    let refusal_input = crate::auto_select_refusal_aggregation::RefusalAggregationInput {
        r_base: base_refusal,
        r_adapt: adapt_res.refusal_code,
        r_dispatch: dispatch_res.refusal_code,
        r_conv: convergence_res.refusal_code,
        r_receipt: ingest_res.refusal_code,
        r_ocel: ocel_res.refusal_code,
        r_trace: trace_res.refusal_code,
        r_epoch: epoch_res.refusal_code,
        m_update: m_update as u8,
    };
    let mut intermediate_refusal = crate::auto_select_refusal_aggregation::aggregate_refusals(&refusal_input);

    // 10. Terminal Convergence (Iteration 41)
    let mut actual_term_input = input.terminal_input;
    actual_term_input.m_tape = tape_mask;
    actual_term_input.r_aggr.critical_refusal = intermediate_refusal;
    
    let term_res = terminal_convergence(&actual_term_input, terminal_state);
    let m_term_commit = m_update_mask & (0u64.wrapping_sub((term_res.refusal_code == 0) as u64));
    
    // For CC=1, we can just replace the terminal state using a mask, or because terminal_convergence 
    // ALREADY returns the masked next_state (it only modifies mass/epoch if admitted).
    // Let's just unconditionally write it since the primitive itself is masked.
    *terminal_state = term_res.next_state;
    
    let final_refusal = intermediate_refusal | term_res.refusal_code;

    FullMapekResult {"""

content = content.replace(old_refusal, new_refusal)

# 5. Result return replacement
content = content.replace(
    "        final_execution_state: dispatch_res.final_state,\n        refusal_code,\n    }",
    "        final_execution_state: dispatch_res.final_state,\n        refusal_code: final_refusal,\n    }"
)

# 6. audit_execute_full_mapek_loop signature
content = content.replace(
    "    trace_state: &mut TraceBufferState<4>,\n) -> FullMapekResult {",
    "    trace_state: &mut TraceBufferState<4>,\n    terminal_state: &mut PersistentControlState,\n) -> FullMapekResult {"
)
content = content.replace(
    "    execute_full_mapek_loop(input, substrate, learning_weights, ocel_state, trace_state)\n}",
    "    execute_full_mapek_loop(input, substrate, learning_weights, ocel_state, trace_state, terminal_state)\n}"
)

# 7. oracle_full_mapek_loop signature
content = content.replace(
    "        ocel_state: &mut OcelBufferState<O>,\n        trace_state: &mut TraceBufferState<P>,\n    ) -> FullMapekResult {",
    "        ocel_state: &mut OcelBufferState<O>,\n        trace_state: &mut TraceBufferState<P>,\n        terminal_state: &mut PersistentControlState,\n    ) -> FullMapekResult {"
)
# oracle also needs to call terminal_convergence? We can just append it.
content = content.replace(
    "        let mut base_refusal = 0;",
    "        let mut base_refusal = 0;\n        let mut term_refusal = 0;"
)
# inside oracle:
content = content.replace(
    "        let refusal_code = crate::auto_select_refusal_aggregation::aggregate_refusals(&refusal_input);\n\n        FullMapekResult {",
    "        let refusal_code = crate::auto_select_refusal_aggregation::aggregate_refusals(&refusal_input);\n\n        let mut term_input = input.terminal_input;\n        term_input.m_tape = tape_mask;\n        term_input.r_aggr.critical_refusal = refusal_code;\n        let term_res = terminal_convergence(&term_input, terminal_state);\n        *terminal_state = term_res.next_state;\n        let final_refusal = refusal_code | term_res.refusal_code;\n\n        FullMapekResult {"
)
content = content.replace(
    "            final_execution_state: dispatch_res.final_state,\n            refusal_code,\n        }",
    "            final_execution_state: dispatch_res.final_state,\n            refusal_code: final_refusal,\n        }"
)

# 8. Mutants signature
old_mut_sig = """    fn mutant_full_bypassed_policy<
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
    ) -> FullMapekResult {"""
new_mut_sig = """    fn mutant_full_bypassed_policy<
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
content = content.replace(old_mut_sig, new_mut_sig)

# same for state_drift mutant
old_mut2_sig = """    fn mutant_full_state_drift<
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
    ) -> FullMapekResult {"""
new_mut2_sig = """    fn mutant_full_state_drift<
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
content = content.replace(old_mut2_sig, new_mut2_sig)

# mutant calls
content = content.replace(
    "execute_full_mapek_loop(&m_input, substrate, learning_weights, ocel_state, trace_state)",
    "execute_full_mapek_loop(&m_input, substrate, learning_weights, ocel_state, trace_state, terminal_state)"
)
content = content.replace(
    "let mut res = execute_full_mapek_loop(input, substrate, learning_weights, ocel_state, trace_state);",
    "let mut res = execute_full_mapek_loop(input, substrate, learning_weights, ocel_state, trace_state, terminal_state);"
)

# test equivalence setup
old_test_setup = """        let mut ocel_state = OcelBufferState::<4>::default();
        let mut trace_state = TraceBufferState::<4>::default();

        let mut oracle_substrate = substrate.clone();"""
new_test_setup = """        let mut ocel_state = OcelBufferState::<4>::default();
        let mut trace_state = TraceBufferState::<4>::default();
        let mut terminal_state = PersistentControlState::default();

        let mut oracle_substrate = substrate.clone();
        let mut oracle_terminal_state = terminal_state.clone();"""
content = content.replace(old_test_setup, new_test_setup)

# test equivalence calls
content = content.replace(
    "execute_full_mapek_loop(&input, &mut substrate, &mut learning_weights, &mut ocel_state, &mut trace_state)",
    "execute_full_mapek_loop(&input, &mut substrate, &mut learning_weights, &mut ocel_state, &mut trace_state, &mut terminal_state)"
)
content = content.replace(
    "oracle_full_mapek_loop(&input, &mut oracle_substrate, &mut oracle_learning_weights, &mut oracle_ocel_state, &mut oracle_trace_state)",
    "oracle_full_mapek_loop(&input, &mut oracle_substrate, &mut oracle_learning_weights, &mut oracle_ocel_state, &mut oracle_trace_state, &mut oracle_terminal_state)"
)
content = content.replace(
    "assert_eq!(trace_state, oracle_trace_state);",
    "assert_eq!(trace_state, oracle_trace_state);\n        assert_eq!(terminal_state, oracle_terminal_state);"
)

# test mutants setup
content = content.replace(
    "let mut input = FullMapekInput::default();",
    "let mut input = FullMapekInput::default();\n        let mut terminal_state = PersistentControlState::default();\n        let mut oracle_terminal_state = terminal_state.clone();"
)
content = content.replace(
    "let oracle_res = oracle_full_mapek_loop(&input, &mut oracle_sub, &mut oracle_lw, &mut oracle_ocel, &mut oracle_trace);",
    "let oracle_res = oracle_full_mapek_loop(&input, &mut oracle_sub, &mut oracle_lw, &mut oracle_ocel, &mut oracle_trace, &mut oracle_terminal_state);"
)
content = content.replace(
    "mutant_full_bypassed_policy(&input, &mut m1_sub, &mut m1_lw, &mut m1_ocel, &mut m1_trace)",
    "mutant_full_bypassed_policy(&input, &mut m1_sub, &mut m1_lw, &mut m1_ocel, &mut m1_trace, &mut terminal_state.clone())"
)
content = content.replace(
    "mutant_full_state_drift(&input, &mut m2_sub, &mut m2_lw, &mut m2_ocel, &mut m2_trace)",
    "mutant_full_state_drift(&input, &mut m2_sub, &mut m2_lw, &mut m2_ocel, &mut m2_trace, &mut terminal_state.clone())"
)

with open(filepath, "w") as f:
    f.write(content)

print("Patched full_mapek_loop.rs")
