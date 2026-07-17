import re

for f in ['crates/bcinr-cmca/tests/differential.rs', 'crates/bcinr-cmca/tests/hostile_mutants.rs', 'crates/bcinr-cmca/tests/case_studies.rs']:
    with open(f, 'r') as file:
        content = file.read()
    
    # Replace NonNegativeFixed(123) with NonNegativeFixed::from_bits(123)
    content = re.sub(r'NonNegativeFixed\(([\d\-]+)\)', r'NonNegativeFixed::from_bits(\1)', content)
    # Replace SignedFixed(123) with SignedFixed::from_bits(123)
    content = re.sub(r'SignedFixed\(([\d\-]+)\)', r'SignedFixed::from_bits(\1)', content)
    
    # Replace .0 with .val ONLY for fixed types, which are mostly like `result[i].0` or `NonNegativeFixed::ONE.0`
    content = re.sub(r'\]\.0', '].val', content)
    content = re.sub(r'::ONE\.0', '::ONE.val', content)
    content = re.sub(r'::ZERO\.0', '::ZERO.val', content)
    
    # Specifically for case_studies expected array size
    content = content.replace("Expected [bcinr_cmca::fixed::NonNegativeFixed; 8]", "Expected [bcinr_cmca::fixed::NonNegativeFixed; 9]")
    
    with open(f, 'w') as file:
        file.write(content)
