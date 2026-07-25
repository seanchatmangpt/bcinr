import re

with open("crates/bcinr-powl/src/mapek_loop.rs", "r") as f:
    content = f.read()

content = content.replace("#[no_mangle]\n#[inline(never)]\npub extern \"C\" fn audit_execute_mapek_loop(\n    input: &MapekInput,\n    substrate: &AutonomicSubstrate<u32, u32, 1>,\n) -> MapekResult<u32, u32, 1> {",
"#[inline(never)]\npub fn audit_execute_mapek_loop(\n    input: &MapekInput,\n    substrate: &mut AutonomicSubstrate<u32, u32, 1>,\n) -> MapekResult {")

with open("crates/bcinr-powl/src/mapek_loop.rs", "w") as f:
    f.write(content)
