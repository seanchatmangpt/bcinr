import re

with open("crates/bcinr-powl/src/mapek_loop.rs", "r") as f:
    code = f.read()

# 1. Imports
code = code.replace(
    "    auto_select_adaptive_mutation::{auto_select_adaptive_mutation, AutoSelectTelemetry},\n    auto_select_substrate_convergence::substrate_convergence,",
    "    auto_select_adaptive_mutation::{auto_select_adaptive_mutation, AutoSelectTelemetry},\n    auto_select_epoch_reclamation::EpochReclamationInput,\n    auto_select_substrate_convergence::substrate_convergence,"
)

# 2. MapekInput struct
code = code.replace(
    "    pub m_outcome: u64,\n}",
    "    pub m_outcome: u64,\n    pub epoch_input: EpochReclamationInput,\n}"
)
code = code.replace(
    "            m_outcome: !0,\n        }",
    "            m_outcome: !0,\n            epoch_input: EpochReclamationInput::default(),\n        }"
)

# 3. MapekResult struct
code = code.replace(
    "pub struct MapekResult {\n    pub tape_mask: u64,\n    pub refusal_code: u8,\n}",
    "pub struct MapekResult {\n    pub tape_mask: u64,\n    pub reclaim_mask: u8,\n    pub refusal_code: u8,\n}"
)

# 4. execute_mapek_loop
old_exec = """    // 6. Substrate Convergence (Iteration 28)
    let convergence_res = substrate_convergence(substrate, &adapt_res.next_state, m_update_mask);

    MapekResult {
        tape_mask,
        refusal_code: base_refusal | convergence_res.refusal_code | adapt_res.refusal_code,
    }"""
new_exec = """    // 6. Substrate Convergence (Iteration 28)
    let convergence_res = substrate_convergence(substrate, &adapt_res.next_state, m_update_mask);

    // 7. Epoch Reclamation (Iteration 30)
    let epoch_res = input.epoch_input.reclaim();
    let reclaim_mask = epoch_res.reclaim_mask & (m_update_mask as u8);

    MapekResult {
        tape_mask,
        reclaim_mask,
        refusal_code: base_refusal | convergence_res.refusal_code | adapt_res.refusal_code | epoch_res.refusal_code,
    }"""
code = code.replace(old_exec, new_exec)

# 5. oracle_mapek_loop
old_oracle_init = """        let mut tape_mask = 0;
        let mut refusal_code = 0;

        let adapt_res = auto_select_adaptive_mutation("""
new_oracle_init = """        let epoch_res = input.epoch_input.reclaim();
        let mut tape_mask = 0;
        let mut reclaim_mask = 0;
        let mut refusal_code = 0;

        let adapt_res = auto_select_adaptive_mutation("""
code = code.replace(old_oracle_init, new_oracle_init)

code = code.replace(
    "        refusal_code |= adapt_res.refusal_code;",
    "        refusal_code |= adapt_res.refusal_code;\n        refusal_code |= epoch_res.refusal_code;"
)

code = code.replace(
    "        if m_update == 1 {\n            substrate.state = adapt_res.next_state;\n            tape_mask = pipeline_res.tape_mask;\n        } else {",
    "        if m_update == 1 {\n            substrate.state = adapt_res.next_state;\n            tape_mask = pipeline_res.tape_mask;\n            reclaim_mask = epoch_res.reclaim_mask;\n        } else {"
)

code = code.replace(
    "        MapekResult {\n            tape_mask,\n            refusal_code,\n        }",
    "        MapekResult {\n            tape_mask,\n            reclaim_mask,\n            refusal_code,\n        }"
)

# 6. mutants
code = code.replace(
    "            m_outcome: input.m_outcome,\n        };",
    "            m_outcome: input.m_outcome,\n            epoch_input: input.epoch_input,\n        };"
)

# 7. tests
old_test_eq = """        assert_eq!(res.tape_mask, 1u64 << 3);
        assert_eq!(res.refusal_code, 0);"""
new_test_eq = """        assert_eq!(res.tape_mask, 1u64 << 3);
        assert_eq!(res.reclaim_mask, 0xFF);
        assert_eq!(res.refusal_code, 0);"""
code = code.replace(old_test_eq, new_test_eq)

with open("crates/bcinr-powl/src/mapek_loop.rs", "w") as f:
    f.write(code)

