import os
import subprocess

for root, _, files in os.walk('crates'):
    for file in files:
        if file.endswith('.rs'):
            path = os.path.join(root, file)
            res = subprocess.run(['rustc', '--crate-type=lib', '--emit=asm', '-O', path], capture_output=True)
            if res.returncode == 0:
                asm = file.replace('.rs', '.s')
                if os.path.exists(asm):
                    with open(asm) as f:
                        content = f.read()
                        if 'panic' in content:
                            print(f'PANIC IN: {path}')
                    os.remove(asm)
