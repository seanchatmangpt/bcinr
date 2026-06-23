import os
import re

algorithms_dir = "crates/bcinr-logic/src/algorithms"
files = [f for f in os.listdir(algorithms_dir) if f.endswith(".rs") and f != "mod.rs"]

for f in files:
    path = os.path.join(algorithms_dir, f)
    with open(path, "r") as file:
        content = file.read()
    
    # Check if it has a reference function for (u64, u64) -> u64
    ref_match = re.search(r'fn ([a-zA-Z0-9_]+)_reference\(\s*val:\s*u64,\s*aux:\s*u64\s*\)\s*->\s*u64', content)
    if not ref_match:
        # try (u32, u32) -> u32
        ref_match = re.search(r'fn ([a-zA-Z0-9_]+)_reference\(\s*val:\s*u32,\s*aux:\s*u32\s*\)\s*->\s*u32', content)
        if not ref_match:
            continue
        typ = "u32"
    else:
        typ = "u64"
        
    func_name = ref_match.group(1)
    
    if "test_exhaustive_fuzz" in content:
        continue # Already injected
        
    # Inject proptest block at the end of the tests module
    # Find the last closing brace of the file
    
    injection = f"""
    use proptest::prelude::*;
    proptest! {{
        #![proptest_config(ProptestConfig::with_cases(10000))]
        #[test]
        fn test_exhaustive_fuzz_{func_name}(val in any::<{typ}>(), aux in any::<{typ}>()) {{
            prop_assert_eq!({func_name}(val, aux), {func_name}_reference(val, aux), "Fuzz mismatch");
        }}
    }}
}}
"""
    
    # Replace the last `}` with the injection
    new_content = content.rstrip()
    if new_content.endswith("}"):
        new_content = new_content[:-1] + injection
        with open(path, "w") as file:
            file.write(new_content)
        print(f"Injected into {f}")

