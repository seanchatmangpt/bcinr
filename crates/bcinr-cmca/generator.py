#!/usr/bin/env python3
import os
import re
import sys

def parse_ttl(file_path):
    classes = {}
    properties = {}
    
    with open(file_path, 'r') as f:
        for line in f:
            # strip comments and whitespace
            line = line.split('#')[0].strip()
            if not line or line.startswith('@prefix'):
                continue
            
            # strip trailing dot
            if line.endswith('.'):
                line = line[:-1].strip()
            
            # parse subject predicate object
            parts = re.split(r'\s+', line, maxsplit=2)
            if len(parts) < 3:
                continue
            
            subj, pred, obj = parts[0], parts[1], parts[2]
            
            # clean value
            val_match = re.match(r'^"([^"]+)"', obj)
            if val_match:
                val_str = val_match.group(1)
                try:
                    if '.' in val_str:
                        val = float(val_str)
                    else:
                        val = int(val_str)
                except ValueError:
                    val = val_str
            else:
                val = obj
                
            if pred == 'a':
                classes[subj] = val
            else:
                if subj not in properties:
                    properties[subj] = {}
                if pred == 'cmca:dependsOn':
                    if 'cmca:dependsOn' not in properties[subj]:
                        properties[subj]['cmca:dependsOn'] = []
                    properties[subj]['cmca:dependsOn'].append(val)
                else:
                    properties[subj][pred] = val
                    
    return classes, properties

def to_q16_16(val):
    scaled = int(round(val * 65536))
    return scaled & 0xFFFFFFFF

def main():
    script_dir = os.path.dirname(os.path.abspath(__file__))
    
    if len(sys.argv) >= 3:
        ttl_path = sys.argv[1]
        out_path = sys.argv[2]
    else:
        ttl_path = os.path.join(script_dir, 'ontology', 'cmca-rdf.ttl')
        out_path = os.path.join(script_dir, 'src', 'generated.rs')
    
    # Ensure target output directory exists
    out_dir = os.path.dirname(out_path)
    if out_dir:
        os.makedirs(out_dir, exist_ok=True)
    
    if not os.path.exists(ttl_path):
        print(f"Error: TTL file not found at {ttl_path}", file=sys.stderr)
        sys.exit(1)
        
    classes, properties = parse_ttl(ttl_path)

    
    # 1. Find all semantic objects
    semantic_objects = [obj for obj, cls in classes.items() if cls == 'cmca:SemanticObject']
    semantic_objects.sort()
    
    # 2. Map dependencies and business values
    dependencies = {}
    business_values = {}
    for obj in semantic_objects:
        props = properties.get(obj, {})
        dependencies[obj] = props.get('cmca:dependsOn', [])
        business_values[obj] = props.get('cmca:businessValue', 0.0)
        
    # 3. Recursively compute downstream consequence mass
    consequence_masses = {}
    memo = {}
    
    def get_consequence_mass(obj, path=None):
        if path is None:
            path = set()
        if obj in memo:
            return memo[obj]
        if obj in path:
            # Simple cycle protection
            return 0.0
        path.add(obj)
        val = business_values.get(obj, 0.0)
        for dep in dependencies.get(obj, []):
            val += get_consequence_mass(dep, path)
        path.remove(obj)
        memo[obj] = val
        return val
        
    for obj in semantic_objects:
        consequence_masses[obj] = get_consequence_mass(obj)
        
    # 4. Generate object registry lines
    object_registry_lines = []
    for i, obj in enumerate(semantic_objects):
        props = properties.get(obj, {})
        recomp = props.get('cmca:recomputationCost', 0.0)
        verify = props.get('cmca:verificationCost', 0.0)
        standing = props.get('cmca:standing', 0.0)
        validity = props.get('cmca:validity', 0.0)
        access = props.get('cmca:accessFrequency', 0.0)
        search = props.get('cmca:searchDemand', 0.0)
        retrieval = props.get('cmca:retrievalDemand', 0.0)
        sched = props.get('cmca:schedulingDemand', 0.0)
        bval = props.get('cmca:businessValue', 0.0)
        conseq = consequence_masses.get(obj, 0.0)
        
        factors = [recomp, verify, standing, validity, access, search, retrieval, sched, bval, conseq]
        factor_names = [
            "recomputationCost", "verificationCost", "standing", "validity",
            "accessFrequency", "searchDemand", "retrievalDemand", "schedulingDemand",
            "businessValue", "downstreamConsequence"
        ]
        
        factor_strs = [f"Fixed({to_q16_16(val)})" for val in factors]
        obj_local_name = obj.split(':')[-1]
        
        lines = [
            f"    // {obj_local_name}",
            "    PackedSemanticState {",
            f"        id: {i},",
            "        factors: [",
        ]
        for f_str, f_name, f_val in zip(factor_strs, factor_names, factors):
            lines.append(f"            {f_str}, // {f_name}: {f_val:.5f}")
        lines.append("        ],"),
        lines.append("    },")
        object_registry_lines.append("\n".join(lines))
        
    # 5. Global ETA
    eta_val = 0.5
    for subj, props in properties.items():
        if 'cmca:eta' in props:
            eta_val = props['cmca:eta']
            break
    eta_q = to_q16_16(eta_val)
    
    # 6. Measure Heads (K)
    measure_heads = [mh for mh, cls in classes.items() if cls == 'cmca:MeasureHead']
    mh_indices = {mh: properties.get(mh, {}).get('cmca:measureIndex', 0) for mh in measure_heads}
    sorted_mh = sorted(measure_heads, key=lambda m: mh_indices[m])
    
    # 7. Lenses (Q)
    lenses = [lens for lens, cls in classes.items() if cls == 'cmca:Lens']
    lens_indices = {lens: properties.get(lens, {}).get('cmca:lensIndex', 0) for lens in lenses}
    lens_exponents = {lens: properties.get(lens, {}).get('cmca:lensExponent', 0.0) for lens in lenses}
    sorted_lenses = sorted(lenses, key=lambda l: lens_indices[l])
    
    # 8. Lambda Matrix
    lambda_coeffs = {}
    for coeff_uri, cls in classes.items():
        if cls == 'cmca:LambdaCoefficient':
            props = properties.get(coeff_uri, {})
            m_idx = props.get('cmca:measureIndex', 0)
            l_idx = props.get('cmca:lensIndex', 0)
            val = props.get('cmca:value', 0.0)
            lambda_coeffs[(m_idx, l_idx)] = val
            
    K = len(sorted_mh)
    Q = len(sorted_lenses)
    
    lambda_matrix_lines = []
    for m_idx in range(K):
        row_strs = []
        for l_idx in range(Q):
            val = lambda_coeffs.get((m_idx, l_idx), 0.0)
            row_strs.append(f"Fixed({to_q16_16(val)})")
        mh_name = sorted_mh[m_idx].split(':')[-1]
        lambda_matrix_lines.append(f"    [{', '.join(row_strs)}], // {mh_name}")
        
    # 9. Lenses spec lines
    lens_registry_lines = []
    for lens in sorted_lenses:
        idx = lens_indices[lens]
        exp = lens_exponents[lens]
        lens_local = lens.split(':')[-1]
        lens_registry_lines.append(
            f"    // {lens_local}\n"
            f"    LensSpec {{ id: {idx}, q: Fixed({to_q16_16(exp)}) }},"
        )
        
    # 10. Write output generated file
    generated_code = f"""// This file is generated by generator.py. DO NOT EDIT.

use crate::fixed::Fixed;

pub const N: usize = {len(semantic_objects)};
pub const F: usize = 10;
pub const K: usize = {K};
pub const Q: usize = {Q};

#[derive(Copy, Clone, Debug)]
pub struct PackedSemanticState {{
    pub id: u32,
    pub factors: [Fixed; F],
}}

#[derive(Copy, Clone, Debug)]
pub struct LensSpec {{
    pub id: u32,
    pub q: Fixed,
}}

pub static ETA: Fixed = Fixed({eta_q}); // {eta_val:.5f}

pub static LAMBDA: [[Fixed; Q]; K] = [
{chr(10).join(lambda_matrix_lines)}
];

pub static OBJECT_REGISTRY: [PackedSemanticState; N] = [
{chr(10).join(object_registry_lines)}
];

pub static LENS_REGISTRY: [LensSpec; Q] = [
{chr(10).join(lens_registry_lines)}
];
"""

    with open(out_path, 'w') as f:
        f.write(generated_code)
        
    print(f"Generated {out_path} successfully (N={len(semantic_objects)}, K={K}, Q={Q}).")

if __name__ == '__main__':
    main()
