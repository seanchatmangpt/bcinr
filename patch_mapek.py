import re

with open("crates/bcinr-powl/src/mapek_loop.rs", "r") as f:
    content = f.read()

# Fix unused mut
content = content.replace(
    "let mut candidate_low = substrate.state.low.saturating_add(input.telemetry_low);",
    "let candidate_low = substrate.state.low.saturating_add(input.telemetry_low);"
)
content = content.replace(
    "let mut candidate_high = substrate.state.high.saturating_add(input.telemetry_high);",
    "let candidate_high = substrate.state.high.saturating_add(input.telemetry_high);"
)

# Fix oracle signature and return
content = content.replace(
    ") -> MapekResult<K, V, N> {",
    ") -> MapekResult {"
)

# Oracle takes &mut
content = content.replace(
    "substrate: &AutonomicSubstrate<K, V, N>,",
    "substrate: &mut AutonomicSubstrate<K, V, N>,"
)

# Replace next_substrate in oracle
content = re.sub(r'let mut next_substrate = \*substrate;\s*let mut tape_mask = 0;\s*let mut refusal_code = 0;',
    'let mut tape_mask = 0;\n        let mut refusal_code = 0;', content)
content = content.replace('next_substrate.state.low = next_substrate.state.low.saturating_add(input.telemetry_low);',
    'substrate.state.low = substrate.state.low.saturating_add(input.telemetry_low);')
content = content.replace('next_substrate.state.high = next_substrate\n                .state\n                .high\n                .saturating_add(input.telemetry_high);',
    'substrate.state.high = substrate.state.high.saturating_add(input.telemetry_high);')
content = content.replace('next_substrate.state.high = next_substrate.state.high.saturating_add(input.telemetry_high);',
    'substrate.state.high = substrate.state.high.saturating_add(input.telemetry_high);')

content = content.replace(
    "MapekResult {\n            next_substrate,\n            tape_mask,\n            refusal_code,\n        }",
    "MapekResult {\n            tape_mask,\n            refusal_code,\n        }"
)

# Fix mutants
content = content.replace(
    "execute_mapek_loop(&m_input, substrate)",
    "execute_mapek_loop(&m_input, substrate)"
)

content = content.replace(
    "res.next_substrate.state.low = substrate.state.low.saturating_add(input.telemetry_low);",
    "substrate.state.low = substrate.state.low.saturating_add(input.telemetry_low);"
)
content = content.replace(
    "res.next_substrate.state.high = substrate.state.high.saturating_add(input.telemetry_high);",
    "substrate.state.high = substrate.state.high.saturating_add(input.telemetry_high);"
)

# Fix tests
content = content.replace(
    "let substrate: AutonomicSubstrate<u32, u32, 1> = AutonomicSubstrate::new();",
    "let mut substrate: AutonomicSubstrate<u32, u32, 1> = AutonomicSubstrate::new();"
)
content = content.replace(
    "let res = execute_mapek_loop(&input, &substrate);",
    "let mut s_res = substrate.clone();\n        let res = execute_mapek_loop(&input, &mut s_res);"
)
content = content.replace(
    "let oracle_res = oracle_mapek_loop(&input, &substrate);",
    "let mut s_oracle = substrate.clone();\n        let oracle_res = oracle_mapek_loop(&input, &mut s_oracle);"
)

content = content.replace("res.next_substrate.state.low", "s_res.state.low")
content = content.replace("oracle_res.next_substrate.state.low", "s_oracle.state.low")
content = content.replace("m2.next_substrate.state.low", "s_m2.state.low")

content = content.replace(
    "let m1 = mutant_mapek_bypassed_policy_guard(&input, &substrate);",
    "let mut s_m1 = substrate.clone();\n        let m1 = mutant_mapek_bypassed_policy_guard(&input, &mut s_m1);"
)
content = content.replace(
    "let m2 = mutant_mapek_state_drift(&input, &substrate);",
    "let mut s_m2 = substrate.clone();\n        let m2 = mutant_mapek_state_drift(&input, &mut s_m2);"
)
content = content.replace(
    "let m3 = mutant_mapek_tape_drift(&input, &substrate);",
    "let mut s_m3 = substrate.clone();\n        let m3 = mutant_mapek_tape_drift(&input, &mut s_m3);"
)

# Check logic for next_substrate.state.high that was wrapped
content = re.sub(r'next_substrate\s*\.state\s*\.high\s*\.saturating_add', r'substrate.state.high.saturating_add', content)


with open("crates/bcinr-powl/src/mapek_loop.rs", "w") as f:
    f.write(content)
