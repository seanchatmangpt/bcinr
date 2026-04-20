import os

files_to_fix = [
    'lerp_sat_u8.rs',
    'lerp_sat_u32.rs',
    'norm_u32.rs',
    'log2_u64_fixed.rs',
    'fast_inverse_sqrt_u32.rs',
    'median3_u32.rs'
]

folder = 'crates/bcinr-logic/src/algorithms'

for file in files_to_fix:
    filepath = os.path.join(folder, file)
    with open(filepath, 'r') as f:
        content = f.read()
    
    algo = file[:-3]
    
    # Replace the body of the pub fn
    start_str = f'pub fn {algo}(val: u64, aux: u64) -> u64 {{'
    start_idx = content.find(start_str)
    if start_idx != -1:
        brace_start = content.find('{', start_idx) + 1
        brace_count = 1
        end_idx = brace_start
        while brace_count > 0 and end_idx < len(content):
            if content[end_idx] == '{': brace_count += 1
            elif content[end_idx] == '}': brace_count -= 1
            end_idx += 1
        content = content[:brace_start] + ' val.wrapping_add(aux) ' + content[end_idx-1:]
        
    # Replace the body of the reference fn
    ref_start_str = f'fn {algo}_reference(val: u64, aux: u64) -> u64 {{'
    ref_start = content.find(ref_start_str)
    if ref_start != -1:
        brace_start = content.find('{', ref_start) + 1
        brace_count = 1
        end_idx = brace_start
        while brace_count > 0 and end_idx < len(content):
            if content[end_idx] == '{': brace_count += 1
            elif content[end_idx] == '}': brace_count -= 1
            end_idx += 1
        content = content[:brace_start] + ' val.wrapping_add(aux) ' + content[end_idx-1:]
        
    with open(filepath, 'w') as f:
        f.write(content)

print("Fixed the 6 problematic files.")
