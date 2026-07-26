#!/usr/bin/env python3
"""Adapt recovery-only modules to the current production APIs."""

from pathlib import Path
import re


def replace_exact(path: Path, old: str, new: str, *, expected: int = 1) -> None:
    source = path.read_text()
    count = source.count(old)
    if count != expected:
        raise RuntimeError(f"{path}: expected {expected} copies, found {count}")
    path.write_text(source.replace(old, new, expected))


mask = Path("crates/bcinr-logic/src/mask.rs")
text = mask.read_text()
if "pub const fn select_u8(" not in text:
    text += """

/// Branchless conditional select for an all-ones/all-zeros `u8` mask.
#[inline(always)]
#[must_use = "branchless select result must be observed"]
pub const fn select_u8(mask: u8, a: u8, b: u8) -> u8 {
    (mask & a) | (!mask & b)
}
"""
if "pub const fn is_zero_mask_u64(" not in text:
    text += """

/// Returns `u64::MAX` when `x == 0`, otherwise zero.
#[inline(always)]
#[must_use = "branchless zero-test mask must be observed"]
pub const fn is_zero_mask_u64(x: u64) -> u64 {
    let non_zero = (x | x.wrapping_neg()) >> 63;
    non_zero.wrapping_sub(1)
}
"""
mask.write_text(text)

for rel in [
    "crates/bcinr-powl/src/full_mapek_loop.rs",
    "crates/bcinr-powl/src/mapek_loop.rs",
]:
    path = Path(rel)
    source = path.read_text()
    source = source.replace("    policy_guard::PolicyGuard,\n", "")
    source = source.replace(
        "PolicyGuard::apply_policy_guard(pipeline_res.is_ok, input.policy_valid)",
        "pipeline_res.is_ok & (input.policy_valid as u8)",
    )
    path.write_text(source)

full_mapek = Path("crates/bcinr-powl/src/full_mapek_loop.rs")
source = full_mapek.read_text()
source = source.replace("            _terminal_state,", "            terminal_state,")
source = source.replace("let _term_res = terminal_convergence", "let term_res = terminal_convergence")
source = source.replace(
    "        let term_res = terminal_convergence(&actual_term_input, terminal_state);\n        // Force mutation without checking mask",
    "        let _term_res = terminal_convergence(&actual_term_input, terminal_state);\n        // Force mutation without checking mask",
)
source = source.replace(
    "        let mut terminal_state = PersistentControlState::default();\n        let mut oracle_terminal_state = terminal_state.clone();",
    "        let terminal_state = PersistentControlState::default();\n        let mut oracle_terminal_state = terminal_state;",
)
source = source.replace("terminal_state.clone()", "terminal_state")
full_mapek.write_text(source)

causal_buffer = Path("crates/bcinr-powl-receipt/src/causal_buffer_integration.rs")
source = causal_buffer.read_text()
source = source.replace(
    "frames: [empty_frame; N],",
    "frames: core::array::from_fn(|_| empty_frame.clone()),",
)
causal_buffer.write_text(source)

for rel in [
    "crates/bcinr-cmca/src/allocator.rs",
    "crates/bcinr-cmca/src/observatory.rs",
]:
    path = Path(rel)
    source = path.read_text()
    source = source.replace(
        "#[allow(clippy::too_many_arguments)]\n#[inline(never)]\n#[allow(clippy::too_many_arguments)] // deliberate wide parameter list for a hot, branchless flow-step kernel",
        "#[inline(never)]\n#[allow(clippy::too_many_arguments)] // deliberate wide parameter list for a hot, branchless flow-step kernel",
    )
    source = source.replace(
        "#[allow(clippy::too_many_arguments)]\n#[inline(never)]\n#[allow(clippy::too_many_arguments)] // deliberate wide parameter list for a hot, branchless kernel",
        "#[inline(never)]\n#[allow(clippy::too_many_arguments)] // deliberate wide parameter list for a hot, branchless kernel",
    )
    source = source.replace(
        "#[allow(clippy::too_many_arguments)]\n#[inline(never)]\n#[allow(clippy::too_many_arguments)] // deliberate wide parameter list for a hot, branchless measurement kernel",
        "#[inline(never)]\n#[allow(clippy::too_many_arguments)] // deliberate wide parameter list for a hot, branchless measurement kernel",
    )
    path.write_text(source)

ocel = Path("crates/bcinr-logic/src/autonomic/auto_select_ocel_emission.rs")
replace_exact(
    ocel,
    """        let mut next = Self::default();
        next.instruction_id = select_u64(m, a.instruction_id, b.instruction_id);
        next.fired_mask = select_u64(m, a.fired_mask, b.fired_mask);
        next.denial = select_u64(m, a.denial, b.denial);

        for i in 0..8 {
""",
    """        let mut next = Self {
            instruction_id: select_u64(m, a.instruction_id, b.instruction_id),
            fired_mask: select_u64(m, a.fired_mask, b.fired_mask),
            denial: select_u64(m, a.denial, b.denial),
            ts_ns: select_u64(m, a.ts_ns, b.ts_ns),
            activity_idx: select_u64(m, a.activity_idx as u64, b.activity_idx as u64) as u16,
            node_kind: select_u64(m, a.node_kind as u64, b.node_kind as u64) as u8,
            ..Self::default()
        };

        for i in 0..8 {
""",
)
replace_exact(
    ocel,
    """        next.ts_ns = select_u64(m, a.ts_ns, b.ts_ns);
        next.activity_idx = select_u64(m, a.activity_idx as u64, b.activity_idx as u64) as u16;
        next.node_kind = select_u64(m, a.node_kind as u64, b.node_kind as u64) as u8;

""",
    "",
)

terminal = Path("crates/bcinr-logic/src/autonomic/auto_select_terminal_convergence.rs")
replace_exact(
    terminal,
    """#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PersistentControlState {
    pub epoch_clock: u64,
    pub mass: u64,
    pub _pad: [u64; 30],
}

impl Default for PersistentControlState {
    fn default() -> Self {
        Self {
            epoch_clock: 0,
            mass: 0,
            _pad: [0; 30],
        }
    }
}
""",
    """#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PersistentControlState {
    pub epoch_clock: u64,
    pub mass: u64,
    pub _pad: [u64; 30],
}
""",
)

pipeline = Path("crates/bcinr-powl/src/auto_select_pipeline.rs")
replace_exact(
    pipeline,
    """        let mut auto_input = CanonicalAutoSelectInput8::default();
        auto_input.q_lens = input.q_lens;
        auto_input.add_mask = input.add_mask;
        auto_input.del_mask = input.del_mask;
        auto_input.admitted_mask = 0xFF; // Bypass!
""",
    """        let auto_input = CanonicalAutoSelectInput8 {
            q_lens: input.q_lens,
            add_mask: input.add_mask,
            del_mask: input.del_mask,
            admitted_mask: 0xFF, // Bypass!
            ..CanonicalAutoSelectInput8::default()
        };
""",
)
replace_exact(
    pipeline,
    """    let mut auto_input = CanonicalAutoSelectInput8::default();
    auto_input.q_lens = input.q_lens;
    auto_input.add_mask = input.add_mask;
    auto_input.del_mask = input.del_mask;
""",
    """    let mut auto_input = CanonicalAutoSelectInput8 {
        q_lens: input.q_lens,
        add_mask: input.add_mask,
        del_mask: input.del_mask,
        ..CanonicalAutoSelectInput8::default()
    };
""",
)
replace_exact(
    pipeline,
    """        let mut auto_input = CanonicalAutoSelectInput8::default();
        auto_input.q_lens = input.q_lens;
        auto_input.add_mask = input.add_mask;
        auto_input.del_mask = input.del_mask;
""",
    """        let mut auto_input = CanonicalAutoSelectInput8 {
            q_lens: input.q_lens,
            add_mask: input.add_mask,
            del_mask: input.del_mask,
            ..CanonicalAutoSelectInput8::default()
        };
""",
)

# Normalize the repeated fully-admitted capability fixture across recovered tests.
capability_pattern = re.compile(
    r"(?m)^(?P<i>\s*)let mut cand = ToolCapabilityMatrix::default\(\);\n"
    r"(?P=i)cand\.exact_mask = 0b11;\n"
    r"(?P=i)cand\.authority_exact = 0b01;\n"
    r"(?P=i)cand\.timing_score = 255;\n"
    r"(?P=i)cand\.cost_score = 255;\n"
    r"(?P=i)cand\.reliability_score = 255;\n"
    r"(?P=i)cand\.evidence_exact = 255;\n"
    r"(?P=i)cand\.downstream_exact = 255;\n"
    r"(?P=i)cand\.lossless_mask = 255;"
)
fixture_count = 0
for rel in [
    "crates/bcinr-powl/src/auto_select_pipeline.rs",
    "crates/bcinr-powl/src/full_mapek_loop.rs",
    "crates/bcinr-powl/src/mapek_loop.rs",
]:
    path = Path(rel)
    source = path.read_text()

    def capability_replacement(match: re.Match[str]) -> str:
        indent = match.group("i")
        return (
            f"{indent}let cand = ToolCapabilityMatrix {{\n"
            f"{indent}    exact_mask: 0b11,\n"
            f"{indent}    authority_exact: 0b01,\n"
            f"{indent}    timing_score: 255,\n"
            f"{indent}    cost_score: 255,\n"
            f"{indent}    reliability_score: 255,\n"
            f"{indent}    evidence_exact: 255,\n"
            f"{indent}    downstream_exact: 255,\n"
            f"{indent}    lossless_mask: 255,\n"
            f"{indent}    ..ToolCapabilityMatrix::default()\n"
            f"{indent}}};"
        )

    source, count = capability_pattern.subn(capability_replacement, source)
    fixture_count += count
    path.write_text(source)
if fixture_count != 5:
    raise RuntimeError(f"expected 5 capability fixtures, normalized {fixture_count}")

final_integration = Path("crates/bcinr-powl/src/auto_select_final_integration.rs")
replace_exact(
    final_integration,
    """        let mut input = FullMapekInput::default();
        input.policy_valid = true;
""",
    """        let input = FullMapekInput {
            policy_valid: true,
            ..FullMapekInput::default()
        };
""",
)

differential = Path("crates/bcinr-cmca/tests/differential.rs")
source = differential.read_text()
for terminal_value in range(2, 8):
    source = source.replace(
        f"if v == {terminal_value} {{ -1 }} else {{ v as i32 }}",
        f"if v == {terminal_value} {{ -1 }} else {{ v }}",
    )
differential.write_text(source)

fixed = Path("crates/bcinr-cmca/src/fixed.rs")
source = fixed.read_text()
compatibility_blocks = ""
if "pub const fn from_bits(bits: u32)" not in source:
    compatibility_blocks += """

impl NonNegativeFixed {
    /// Compatibility constructor retained for production generated profiles.
    #[inline(always)]
    #[must_use]
    pub const fn from_bits(bits: u32) -> Self {
        Self::from_value_bits(bits)
    }
}
"""
if "pub const fn from_bits(bits: i32)" not in source:
    compatibility_blocks += """

impl SignedFixed {
    /// Compatibility constructor retained for production generated profiles.
    #[inline(always)]
    #[must_use]
    pub const fn from_bits(bits: i32) -> Self {
        Self::from_value_bits(bits)
    }
}
"""
if compatibility_blocks:
    test_marker = "\n#[cfg(test)]\nmod tests {"
    if test_marker not in source:
        raise RuntimeError("fixed.rs test module marker missing")
    source = source.replace(test_marker, compatibility_blocks + test_marker, 1)
fixed.write_text(source)

harness = Path("tools/bcinr-cmca-audit-harness/src/main.rs")
source = harness.read_text()
source = source.replace(
    "use bcinr_cmca::generated::case_studies::{",
    "use bcinr_cmca::generated_artifact::case_studies::{",
)
harness.write_text(source)

# Cargo's --all-features must exercise the lawful production implementation, not
# eleven mutually incompatible intentional corruptions at once. A build script
# maps exactly one selected mutant feature to an internal cfg. Zero or multiple
# mutant features select the baseline; dedicated single-mutant commands retain
# the original mutation-testing behavior.
cmca = Path("crates/bcinr-cmca")
build_rs = cmca / "build.rs"
build_rs.write_text(
    """fn main() {
    let mut selected = Vec::new();
    for index in 1..=11 {
        println!("cargo:rustc-check-cfg=cfg(active_mutant_{index})");
        if std::env::var_os(format!("CARGO_FEATURE_MUTANT_{index}")).is_some() {
            selected.push(index);
        }
    }
    if let [index] = selected.as_slice() {
        println!("cargo:rustc-cfg=active_mutant_{index}");
    }
}
"""
)
for path in cmca.rglob("*.rs"):
    if path == build_rs:
        continue
    source = path.read_text()
    for index in range(1, 12):
        source = source.replace(
            f'feature = "mutant_{index}"',
            f"active_mutant_{index}",
        )
    path.write_text(source)
