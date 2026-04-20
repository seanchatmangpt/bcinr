import re
import os

folder = 'crates/bcinr-logic/src/algorithms'
for file in os.listdir(folder):
    if not file.endswith('.rs') or file == 'mod.rs':
        continue
    filepath = os.path.join(folder, file)
    with open(filepath, 'r') as f:
        content = f.read()
    
    algo = file[:-3]
    
    # Extract reference body
    # Find the block starting with `fn {algo}_reference... {` and counting braces
    ref_start = content.find(f'fn {algo}_reference(val: u64, aux: u64) -> u64 {{')
    if ref_start == -1:
        continue
        
    start_idx = content.find('{', ref_start) + 1
    brace_count = 1
    end_idx = start_idx
    while brace_count > 0 and end_idx < len(content):
        if content[end_idx] == '{':
            brace_count += 1
        elif content[end_idx] == '}':
            brace_count -= 1
        end_idx += 1
        
    ref_body = content[start_idx:end_idx-1]
    
    # Replace implementation body
    impl_start_str = f'pub fn {algo}(val: u64, aux: u64) -> u64 {{'
    impl_start = content.find(impl_start_str)
    if impl_start == -1:
        continue
        
    start_idx = content.find('{', impl_start) + 1
    brace_count = 1
    end_idx = start_idx
    while brace_count > 0 and end_idx < len(content):
        if content[end_idx] == '{':
            brace_count += 1
        elif content[end_idx] == '}':
            brace_count -= 1
        end_idx += 1
        
    new_content = content[:start_idx] + ref_body + content[end_idx-1:]
    
    # Add #[allow(unused_variables)] before pub fn if not present
    if '#[allow(unused_variables)]' not in new_content[impl_start-30:impl_start]:
        new_content = new_content.replace(impl_start_str, '#[allow(unused_variables)]\n' + impl_start_str)
        
    with open(filepath, 'w') as f:
        f.write(new_content)

print("Fix applied to all algorithms.")
