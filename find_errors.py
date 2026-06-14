import subprocess
import re

result = subprocess.run(['cargo', 'check', '--lib'], stderr=subprocess.PIPE, text=True)
output = result.stderr

errors = re.findall(r'error\[?.*?\]?: .*?\n  --> (.*?):(\d+):(\d+)', output)

for file, line, col in errors:
    print(f"Error in {file} at {line}:{col}")

# Also catch errors without [E...] codes
errors2 = re.findall(r'error: .*?\n  --> (.*?):(\d+):(\d+)', output)
for file, line, col in errors2:
    print(f"Error in {file} at {line}:{col}")
