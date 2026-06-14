import os
import re
import subprocess

DIR = "crates/bcinr-logic/src/algorithms"
files = sorted(os.listdir(DIR))
start = files.index('morton_decode_2d_u32.rs')
end = files.index('quadtree_insert_branchless.rs')
batch_files = files[start:end+1]

def unroll_parallel_bits_deposit():
    path = os.path.join(DIR, "parallel_bits_deposit_u64.rs")
    with open(path, "r") as f:
        content = f.read()

    unrolled_body = ""
    for i in range(64):
        unrolled_body += """        let m_bit_{i} = m & 1;
        let v_bit_{i} = v & 1;
        res |= (m_bit_{i} & v_bit_{i}).wrapping_mul(pos);
        v >>= m_bit_{i};
        m >>= 1;
        pos <<= 1;
""".format(i=i)

    # Replace the while loop in both the main function and reference function
    loop_regex = re.compile(r"let mut i = 0;\s*while i < 64 \{.*?i \+= 1;\s*\}", re.DOTALL)
    
    new_content = loop_regex.sub(unrolled_body.rstrip(), content)
    with open(path, "w") as f:
        f.write(new_content)

def replace_operators():
    for f in batch_files:
        path = os.path.join(DIR, f)
        with open(path, "r") as f_in:
            lines = f_in.readlines()
        
        out_lines = []
        for line in lines:
            if line.strip().startswith("//") or line.strip().startswith("#") or "mutant" in line or "test_" in line or "-> u64 {" in line or "fn " in line:
                out_lines.append(line)
                continue
            
            # Simple replacements for binary operators not in macros or strings
            # Only touching things that look like math on variables
            # Also avoiding cases where it's already wrapping_add
            # Or pointers, deref, etc.
            
            # Very cautious replace:
            # We want to replace `a + b` with `a.wrapping_add(b)`.
            # A simple approach for standard operators (+, -, *) 
            # with spaces around them:
            if " + " in line and ".wrapping_add" not in line and 'assert' not in line:
                line = re.sub(r'([a-zA-Z0-9_]+)\s*\+\s*([a-zA-Z0-9_]+|0x[0-9A-Fa-f]+)', r'\1.wrapping_add(\2)', line)
            if " - " in line and ".wrapping_sub" not in line and 'assert' not in line:
                line = re.sub(r'([a-zA-Z0-9_]+)\s*-\s*([a-zA-Z0-9_]+|0x[0-9A-Fa-f]+)', r'\1.wrapping_sub(\2)', line)
            if " * " in line and ".wrapping_mul" not in line and 'assert' not in line:
                line = re.sub(r'([a-zA-Z0-9_]+)\s*\*\s*([a-zA-Z0-9_]+|0x[0-9A-Fa-f]+)', r'\1.wrapping_mul(\2)', line)
            
            out_lines.append(line)
            
        with open(path, "w") as f_out:
            f_out.writelines(out_lines)

unroll_parallel_bits_deposit()
replace_operators()

print("Stabilization complete.")
