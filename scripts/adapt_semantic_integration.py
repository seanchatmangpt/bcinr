#!/usr/bin/env python3
"""Adapt recovery-only modules to the current production APIs."""

from pathlib import Path

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
    "        let terminal_state = PersistentControlState::default();\n        let mut oracle_terminal_state = terminal_state.clone();",
)
full_mapek.write_text(source)

causal_buffer = Path("crates/bcinr-powl-receipt/src/causal_buffer_integration.rs")
source = causal_buffer.read_text()
source = source.replace(
    "frames: [empty_frame; N],",
    "frames: core::array::from_fn(|_| empty_frame.clone()),",
)
causal_buffer.write_text(source)

fixed = Path("crates/bcinr-cmca/src/fixed.rs")
source = fixed.read_text()
if "pub const fn from_bits(bits: u32)" not in source:
    source += """

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
    source += """

impl SignedFixed {
    /// Compatibility constructor retained for production generated profiles.
    #[inline(always)]
    #[must_use]
    pub const fn from_bits(bits: i32) -> Self {
        Self::from_value_bits(bits)
    }
}
"""
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
