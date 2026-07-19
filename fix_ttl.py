import os
import re

measures = {
    "0": "cmca:MeasureCache",
    "1": "cmca:MeasureSearch",
    "2": "cmca:MeasureRetrieval",
    "3": "cmca:MeasureScheduling"
}

lenses = {
    "0": "cmca:LensExploitation",
    "1": "cmca:LensProportional",
    "2": "cmca:LensCoverage",
    "3": "cmca:LensRare"
}

for filename in ["crates/bcinr-cmca/ontology/cmca-rdf.ttl", "crates/bcinr-cmca/ontology/generalization.ttl"]:
    with open(filename, 'r') as f:
        content = f.read()
    
    # Replace cmca:measureIndex "X"^^xsd:integer with cmca:measure cmca:Measure...
    def replace_measure(m):
        idx = m.group(1)
        return f'cmca:measure {measures[idx]}'
    content = re.sub(r'cmca:measureIndex\s+"(\d+)"\^\^xsd:integer', replace_measure, content)

    # Replace cmca:lensIndex "X"^^xsd:integer with cmca:lens cmca:Lens...
    def replace_lens(m):
        idx = m.group(1)
        return f'cmca:lens {lenses[idx]}'
    content = re.sub(r'cmca:lensIndex\s+"(\d+)"\^\^xsd:integer', replace_lens, content)

    with open(filename, 'w') as f:
        f.write(content)
