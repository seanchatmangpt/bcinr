with open("crates/bcinr-powl/src/full_mapek_loop.rs", "r") as f:
    text = f.read()

text = text.replace("_terminal_state: &mut PersistentControlState", "terminal_state: &mut PersistentControlState")
text = text.replace("let terminal_state = PersistentControlState::default();", "let mut terminal_state = PersistentControlState::default();")
text = text.replace("let term1 = terminal_state.clone();", "let mut term1 = terminal_state.clone();")
text = text.replace("let term2 = terminal_state.clone();", "let mut term2 = terminal_state.clone();")
text = text.replace("let term3 = terminal_state.clone();", "let mut term3 = terminal_state.clone();")
text = text.replace("let term4 = terminal_state.clone();", "let mut term4 = terminal_state.clone();")
text = text.replace("&mut terminal_state.clone()", "&mut terminal_state")

with open("crates/bcinr-powl/src/full_mapek_loop.rs", "w") as f:
    f.write(text)

with open("crates/bcinr-powl/src/auto_select_final_integration.rs", "r") as f:
    text = f.read()

text = text.replace("let mut terminal_state = PersistentControlState::default();\n        let _ref_res =", "let mut terminal_state = PersistentControlState::default();\n        let mut w1 = LearningWeights::default();\n        let _ref_res =")

text = text.replace(
"""        let _ref_res =
            final_integration_reference(&input, &mut sub_ref, &mut w_ref, &mut o_ref, &mut _t_ref, &mut terminal_state);""",
"""        let mut terminal_state = PersistentControlState::default();
        let _ref_res =
            final_integration_reference(&input, &mut sub_ref, &mut w_ref, &mut o_ref, &mut _t_ref, &mut terminal_state);"""
)

with open("crates/bcinr-powl/src/auto_select_final_integration.rs", "w") as f:
    f.write(text)

