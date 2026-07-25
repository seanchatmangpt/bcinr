import re
with open("crates/bcinr-powl/src/auto_select_pipeline.rs", "r") as f:
    content = f.read()

# Replace all powl_bridge_select calls to use canonical_res directly
content = re.sub(
    r'powl_bridge_select\(&bridge_auto_res\)',
    'powl_bridge_select(&canonical_res)',
    content
)

# And remove let bridge_auto_res = ... entirely
# This struct init can be multiline, so we need to match carefully
content = re.sub(
    r'let bridge_auto_res = bcinr_logic::autonomic::(?:canonical_mass::)?AutoSelectResult\s*\{[^}]+\};',
    '',
    content
)

# Also fix the unused variable auto_res if it exists
content = re.sub(
    r'powl_bridge_select\(&auto_res\)',
    'powl_bridge_select(&canonical_res)',
    content
)

# Also remove auto_res definition
content = re.sub(
    r'let auto_res = bcinr_logic::autonomic::(?:canonical_mass::)?AutoSelectResult\s*\{[^}]+\};',
    '',
    content
)

with open("crates/bcinr-powl/src/auto_select_pipeline.rs", "w") as f:
    f.write(content)
