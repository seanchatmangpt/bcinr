import os
import re

def fix_black_box(file_path):
    with open(file_path, 'r') as f:
        content = f.read()
    
    # Replace black_box(a, b, c) with black_box(a), black_box(b), black_box(c)
    content = re.sub(r'black_box\(([^,)]+),\s*([^,)]+),\s*([^,)]+)\)', r'black_box(\1), black_box(\2), black_box(\3)', content)
    
    # Replace black_box(a, b) with black_box(a), black_box(b)
    content = re.sub(r'black_box\(([^,)]+),\s*([^,)]+)\)', r'black_box(\1), black_box(\2)', content)
    
    with open(file_path, 'w') as f:
        f.write(content)

algorithms_dir = 'crates/bcinr-logic/src/algorithms'
for filename in os.listdir(algorithms_dir):
    if filename.endswith('.rs'):
        fix_black_box(os.path.join(algorithms_dir, filename))

print("Fixed algorithm files (2 and 3 arguments).")

all_300_bench = 'bcinr-bench/benches/all_300_bench.rs'
if os.path.exists(all_300_bench):
    fix_black_box(all_300_bench)
    print(f"Fixed {all_300_bench}")
