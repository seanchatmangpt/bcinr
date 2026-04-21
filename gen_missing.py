import os
import re
import subprocess

proc = subprocess.run(['cargo', 'run', '--package', 'bcinr-bench-auditor'], capture_output=True, text=True)
missing = []
for line in proc.stdout.splitlines():
    if line.startswith('  - '):
        missing.append(line.strip()[2:])

def get_sig(fn):
    for root, _, files in os.walk('crates/bcinr-logic/src'):
        for f in files:
            if f.endswith('.rs'):
                path = os.path.join(root, f)
                content = open(path).read()
                # find pub fn fn_name(args)
                m = re.search(r'pub\s+(?:const\s+)?fn\s+' + fn + r'\s*(<.*?>)?\s*\((.*?)\)', content, re.DOTALL)
                if m:
                    mod = path.replace('crates/bcinr-logic/src/', '').replace('.rs', '').replace('/', '::')
                    if mod == 'lib': mod = ''
                    return mod, m.group(2).strip()
    return None, None

def arg_to_vals(atype):
    atype = atype.strip()
    if 'u64' in atype and '&' not in atype and '[' not in atype: return ('0u64', 'u64::MAX', '42u64')
    if 'u32' in atype and '&' not in atype and '[' not in atype: return ('0u32', 'u32::MAX', '42u32')
    if 'u16' in atype and '&' not in atype and '[' not in atype: return ('0u16', 'u16::MAX', '42u16')
    if 'u8' in atype and '&' not in atype and '[' not in atype: return ('0u8', 'u8::MAX', '42u8')
    if 'i64' in atype and '&' not in atype and '[' not in atype: return ('0i64', 'i64::MAX', '42i64')
    if 'i32' in atype and '&' not in atype and '[' not in atype: return ('0i32', 'i32::MAX', '42i32')
    if 'usize' in atype and '&' not in atype and '[' not in atype: return ('0usize', 'usize::MAX', '42usize')
    if 'f32' in atype and '&' not in atype and '[' not in atype: return ('0f32', 'f32::MAX', '42.0f32')
    
    if '&[u8]' in atype: return ('[0u8; 32]', '[255u8; 32]', '[42u8; 32]')
    if '&[u32]' in atype: return ('[0u32; 16]', '[u32::MAX; 16]', '[42u32; 16]')
    if '&[usize]' in atype: return ('[0usize; 16]', '[usize::MAX; 16]', '[42usize; 16]')
    if '&[u64]' in atype: return ('[0u64; 16]', '[u64::MAX; 16]', '[42u64; 16]')
    
    if '&mut [u64]' in atype: return ('[0u64; 16]', '[u64::MAX; 16]', '[42u64; 16]')
    if '&mut [usize]' in atype: return ('[0usize; 16]', '[usize::MAX; 16]', '[42usize; 16]')
    if '&mut [u32]' in atype: return ('[0u32; 16]', '[u32::MAX; 16]', '[42u32; 16]')
    
    if '&mut [u32; 8]' in atype: return ('[0u32; 8]', '[u32::MAX; 8]', '[42u32; 8]')
    if '&mut [u32; 32]' in atype: return ('[0u32; 32]', '[u32::MAX; 32]', '[42u32; 32]')
    if '&mut [u32; 16]' in atype: return ('[0u32; 16]', '[u32::MAX; 16]', '[42u32; 16]')
    
    if '[u8; 16]' in atype: return ('[0u8; 16]', '[255u8; 16]', '[42u8; 16]')
    if '[usize]' in atype: return ('[0usize; 16]', '[usize::MAX; 16]', '[42usize; 16]')
    if '[u64]' in atype: return ('[0u64; 16]', '[u64::MAX; 16]', '[42u64; 16]')
    if '[u32]' in atype: return ('[0u32; 16]', '[u32::MAX; 16]', '[42u32; 16]')
    
    return ('0', '0', '0') # fallback

out = "use bcinr_logic::*;\nuse criterion::{criterion_group, criterion_main, Criterion, black_box};\n\n"

for fn in missing:
    mod, args = get_sig(fn)
    if mod is None: continue
    
    prefix = f"{mod}::" if mod else ""
    if prefix.startswith('algorithms::'):
        prefix = f"algorithms::{fn}::"
    
    out += f"fn bench_{fn}(c: &mut Criterion) {{\n"
    
    variants = [("min", 0), ("max", 1), ("avg", 2)]
    
    for variant_name, idx in variants:
        call_args = []
        let_bindings = ""
        for i, arg in enumerate(args.split(',')):
            if not arg.strip(): continue
            parts = arg.split(':')
            if len(parts) >= 2:
                atype = parts[1].strip()
                vals = arg_to_vals(atype)
                val_base = vals[idx]
                
                if '&mut ' in atype:
                    let_bindings += f"        let mut arg{i}_{variant_name} = {val_base};\n"
                    call_args.append(f"black_box(&mut arg{i}_{variant_name})")
                elif '&' in atype:
                    let_bindings += f"        let arg{i}_{variant_name} = {val_base};\n"
                    call_args.append(f"black_box(&arg{i}_{variant_name})")
                else:
                    call_args.append(f"black_box({val_base})")

        call_str = ", ".join(call_args)
        
        out += f"    c.bench_function(\"{fn}_{variant_name}\", |b| {{\n"
        out += let_bindings
        out += f"        b.iter(|| {{\n"
        out += f"            {prefix}{fn}({call_str})\n"
        out += f"        }})\n"
        out += f"    }});\n"
        
    out += f"}}\n\n"

out += "criterion_group!(benches,\n"
for fn in missing:
    out += f"    bench_{fn},\n"
out += ");\n"
out += "criterion_main!(benches);\n"

open('bcinr-bench/benches/missing_bench.rs', 'w').write(out)
print("Generated missing_bench.rs with min/max payloads.")
