with open('crates/bcinr-cmca/generator.py', 'r') as f:
    content = f.read()

patch = """
            if not m_uri and 'cmca:measureIndex' in props:
                m_idx = int(props['cmca:measureIndex'])
                if m_idx < len(sorted_mh):
                    m_uri = sorted_mh[m_idx]
            if not l_uri and 'cmca:lensIndex' in props:
                l_idx = int(props['cmca:lensIndex'])
                if l_idx < len(sorted_lenses):
                    l_uri = sorted_lenses[l_idx]
"""

content = content.replace("m_uri = props.get('cmca:measure')\n            l_uri = props.get('cmca:lens')", "m_uri = props.get('cmca:measure')\n            l_uri = props.get('cmca:lens')" + patch)

with open('crates/bcinr-cmca/generator.py', 'w') as f:
    f.write(content)
