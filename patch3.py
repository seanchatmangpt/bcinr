with open('crates/bcinr-cmca/generator.py', 'r') as f:
    content = f.read()

import re

# Patch Lenses sorting
content = re.sub(
    r"lenses = \[lens for lens, cls in classes.items\(\) if cls == 'cmca:Lens'\]\s+lens_exponents = \{lens: properties.get\(lens, \{\}\).get\('cmca:lensExponent', 0.0\) for lens in lenses\}\s+sorted_lenses = sorted\(lenses\)",
    "lenses = [lens for lens, cls in classes.items() if cls == 'cmca:Lens']\n    lens_indices = {lens: properties.get(lens, {}).get('cmca:lensIndex', 0) for lens in lenses}\n    lens_exponents = {lens: properties.get(lens, {}).get('cmca:lensExponent', 0.0) for lens in lenses}\n    sorted_lenses = sorted(lenses, key=lambda l: int(lens_indices[l]))",
    content
)

with open('crates/bcinr-cmca/generator.py', 'w') as f:
    f.write(content)
