import re

with open('crates/bcinr-logic/src/algorithms/parallel_bits_deposit_u64.rs', 'r') as f:
    content = f.read()

unrolled = "    let mut res = 0u64;\n    let mut v = val;\n    let mut m = aux;\n    let mut pos = 1u64;\n"
for i in range(64):
    unrolled += f"""    let m_bit_{i} = m & 1;
    let v_bit_{i} = v & 1;
    res |= (m_bit_{i} & v_bit_{i}).wrapping_mul(pos);
    v >>= m_bit_{i};
    m >>= 1;
    pos <<= 1;
"""

new_impl = f"""#[no_mangle]
#[allow(unused_variables)]
pub fn parallel_bits_deposit_u64(val: u64, aux: u64) -> u64 {{
{unrolled}
    res
}}"""

# Find pub fn
start = content.find('#[no_mangle]\n#[allow(unused_variables)]\npub fn parallel_bits_deposit_u64')
end = content.find('}', content.find('}', start)+1) + 1 # wait, this finds the end of the fn. It's safer to use regex.

content = re.sub(r'#\[no_mangle\]\n#\[allow\(unused_variables\)\]\npub fn parallel_bits_deposit_u64.*?\}\n\n', new_impl + "\n\n", content, flags=re.DOTALL)

with open('crates/bcinr-logic/src/algorithms/parallel_bits_deposit_u64.rs', 'w') as f:
    f.write(content)

