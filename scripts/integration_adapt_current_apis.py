#!/usr/bin/env python3
"""Narrow adapters for recovery modules against the current main APIs."""

from pathlib import Path

ROOT = Path.cwd()

mask = ROOT / "crates/bcinr-logic/src/mask.rs"
text = mask.read_text()
if "pub const fn select_u8(" not in text:
    text += '''

/// Branchless conditional select for an all-ones/all-zeros `u8` mask.
#[inline(always)]
#[must_use = "branchless select — ignoring this result discards the computed selection"]
pub const fn select_u8(mask: u8, a: u8, b: u8) -> u8 {
    (mask & a) | (!mask & b)
}
'''
if "pub const fn is_zero_mask_u64(" not in text:
    text += '''

/// Branchless zero-test mask for `u64` values.
#[inline(always)]
#[must_use = "branchless zero mask — ignoring this result discards the zero-test"]
pub const fn is_zero_mask_u64(x: u64) -> u64 {
    let non_zero_msb = (x | x.wrapping_neg()) >> 63;
    non_zero_msb.wrapping_sub(1)
}
'''
mask.write_text(text)

for rel in (
    "crates/bcinr-powl/src/mapek_loop.rs",
    "crates/bcinr-powl/src/full_mapek_loop.rs",
):
    path = ROOT / rel
    source = path.read_text()
    source = source.replace("    policy_guard::PolicyGuard,\n", "")
    source = source.replace(
        "PolicyGuard::apply_policy_guard(pipeline_res.is_ok, input.policy_valid)",
        "pipeline_res.is_ok & 0u8.wrapping_sub(input.policy_valid as u8)",
    )
    path.write_text(source)
