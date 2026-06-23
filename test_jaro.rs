fn jaro_winkler_branchless(val: u64, aux: u64) -> u64 {
    const H: u64 = 0x8080808080808080;
    const LO7: u64 = 0x7F7F7F7F7F7F7F7F;
    let x = val ^ aux; // zero byte == positional match
    let t = (x & LO7).wrapping_add(LO7);
    let zb = !(t | x) & H; // high bit set per matching byte
    let m = zb.count_ones() as u64; // number of matching positions
    let nz = H & !zb; // high bit set per mismatching byte
    let p = ((nz.trailing_zeros() as u64) >> 3).min(4); // capped common prefix
    m.wrapping_mul(125).wrapping_add(p.wrapping_mul(10))
}

fn jaro_winkler_branchless_reference(val: u64, aux: u64) -> u64 {
    let a = val.to_le_bytes();
    let b = aux.to_le_bytes();
    let mut m: u64 = 0;
    for i in 0..8 {
        if a[i] == b[i] {
            m += 1;
        }
    }
    let mut p: u64 = 0;
    for i in 0..8 {
        if a[i] == b[i] && p < 4 {
            p += 1;
        } else {
            break;
        }
    }
    m * 125 + p * 10
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
        if jaro_winkler_branchless(v, a) != jaro_winkler_branchless_reference(v, a) {
            println!("BUG FOUND in jaro: v={}, a={}", v, a);
            return;
        }
    }
    
    let v = 0x0101010101010101;
    let a = 0x0101010101010101;
    if jaro_winkler_branchless(v, a) != jaro_winkler_branchless_reference(v, a) {
        println!("BUG FOUND in jaro: v={}, a={}", v, a);
        return;
    }
    
    println!("jaro OK");
}
