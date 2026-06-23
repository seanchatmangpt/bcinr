fn mismatch_branchless_u8(val: u64, aux: u64) -> u64 {
    let diff = val ^ aux;
    let has_diff = (diff != 0) as u64;
    let pos = (diff.trailing_zeros() >> 3) as u64;
    pos * has_diff + 8 * (1 - has_diff)
}

fn mismatch_branchless_u8_reference(val: u64, aux: u64) -> u64 {
    let a = val.to_le_bytes();
    let b = aux.to_le_bytes();
    for i in 0..8 {
        if a[i] != b[i] {
            return i as u64;
        }
    }
    8
}

fn main() {
    let mut rng = 1337u64;
    let mut next_u64 = || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };
    
    for _ in 0..1000000 {
        let v = next_u64();
        let a = next_u64();
        if mismatch_branchless_u8(v, a) != mismatch_branchless_u8_reference(v, a) {
            println!("BUG FOUND in mismatch: v={}, a={}", v, a);
            return;
        }
    }
    println!("mismatch OK");
}
