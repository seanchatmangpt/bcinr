import sys

path = "crates/bcinr-logic/src/simd_dispatch.rs"
with open(path, "r") as f:
    lines = f.readlines()

for i, line in enumerate(lines):
    if "mod tests {" in line:
        tests_idx = i
        break

# Code to insert
injections = """
// ---------------------------------------------------------------------------
// SIMDe Philosophy Extensions: Blend, Shift, PDEP/PEXT
// ---------------------------------------------------------------------------

#[allow(dead_code)]
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse4.2,ssse3")]
unsafe fn blend_u8x16_sse(mask: [u8; 16], a: [u8; 16], b: [u8; 16]) -> [u8; 16] {
    use core::arch::x86_64::*;
    let vm = _mm_loadu_si128(mask.as_ptr() as *const __m128i);
    let va = _mm_loadu_si128(a.as_ptr() as *const __m128i);
    let vb = _mm_loadu_si128(b.as_ptr() as *const __m128i);
    let result = _mm_blendv_epi8(va, vb, vm); // if mask MSB=1 select vb else va
    let mut out = [0u8; 16];
    _mm_storeu_si128(out.as_mut_ptr() as *mut __m128i, result);
    out
}

#[allow(dead_code)]
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn blend_u8x16_neon(mask: [u8; 16], a: [u8; 16], b: [u8; 16]) -> [u8; 16] {
    use core::arch::aarch64::*;
    // NEON vbslq_u8 selects bits directly based on mask bits
    // x86 blendv checks MSB of mask bytes. To match semantics branchlessly, we extend MSB to full byte.
    let vm = vld1q_u8(mask.as_ptr());
    let mask_msb = vshrq_n_s8(vreinterpretq_s8_u8(vm), 7);
    let vm_full = vreinterpretq_u8_s8(mask_msb);
    let va = vld1q_u8(a.as_ptr());
    let vb = vld1q_u8(b.as_ptr());
    let result = vbslq_u8(vm_full, vb, va);
    let mut out = [0u8; 16];
    vst1q_u8(out.as_mut_ptr(), result);
    out
}

#[inline(always)]
fn blend_u8x16_scalar(mask: [u8; 16], a: [u8; 16], b: [u8; 16]) -> [u8; 16] {
    let mut out = [0u8; 16];
    let mut i = 0usize;
    while i < 16 {
        let msb = (mask[i] >> 7) as u8;
        let m = msb.wrapping_neg(); // 0xFF or 0x00
        out[i] = (b[i] & m) | (a[i] & !m);
        i += 1;
    }
    out
}

#[inline(always)]
pub fn blend_u8x16(mask: [u8; 16], a: [u8; 16], b: [u8; 16]) -> [u8; 16] {
    #[cfg(all(target_arch = "x86_64", target_feature = "ssse3"))]
    { unsafe { blend_u8x16_sse(mask, a, b) } }
    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    { unsafe { blend_u8x16_neon(mask, a, b) } }
    #[cfg(not(any(
        all(target_arch = "x86_64", target_feature = "ssse3"),
        all(target_arch = "aarch64", target_feature = "neon"),
    )))]
    { blend_u8x16_scalar(mask, a, b) }
}

#[inline(always)]
pub fn pdep_u64(val: u64, mask: u64) -> u64 {
    // Branchless software PDEP
    let mut res = 0u64;
    let mut m = mask;
    let mut v = val;
    let mut i = 0;
    while i < 64 {
        let lowest_mask_bit = m & m.wrapping_neg();
        let cond = (lowest_mask_bit != 0) as u64;
        let v_bit = v & 1;
        res |= (v_bit * lowest_mask_bit) * cond;
        v >>= cond;
        m ^= lowest_mask_bit;
        i += 1;
    }
    res
}

#[inline(always)]
pub fn pext_u64(val: u64, mask: u64) -> u64 {
    // Branchless software PEXT
    let mut res = 0u64;
    let mut m = mask;
    let mut shift = 0;
    let mut i = 0;
    while i < 64 {
        let lowest_mask_bit = m & m.wrapping_neg();
        let cond = (lowest_mask_bit != 0) as u64;
        let bit = (val & lowest_mask_bit) != 0;
        res |= (bit as u64) << shift;
        shift += cond;
        m ^= lowest_mask_bit;
        i += 1;
    }
    res
}

"""

lines.insert(tests_idx - 1, injections)

test_injections = """
    #[test]
    fn test_blend_u8x16() {
        let mut a = [0u8; 16];
        let mut b = [0u8; 16];
        let mut mask = [0u8; 16];
        a[0] = 1; a[1] = 2; a[2] = 3;
        b[0] = 10; b[1] = 20; b[2] = 30;
        mask[1] = 0x80; // select b for index 1
        mask[2] = 0x7F; // select a for index 2 (MSB=0)
        let r = blend_u8x16(mask, a, b);
        assert_eq!(r[0], 1);
        assert_eq!(r[1], 20);
        assert_eq!(r[2], 3);
    }
    #[test]
    fn test_pdep_pext() {
        assert_eq!(pdep_u64(0b101, 0b01010100), 0b01000100);
        assert_eq!(pext_u64(0b11010110, 0b01010100), 0b101);
    }
"""

with open(path, "w") as f:
    f.writelines(lines)

with open(path, "r") as f:
    text = f.read()

text = text.replace("mod tests {", "mod tests {\n" + test_injections)

with open(path, "w") as f:
    f.write(text)

