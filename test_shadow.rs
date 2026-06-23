fn levenshtein_dist_branchless(val: u64, aux: u64) -> u64 {
    let diff = val ^ aux;
    const LO: u64 = 0x0101_0101_0101_0101;
    const HI: u64 = 0x8080_8080_8080_8080;
    let nonzero = (((diff | HI).wrapping_sub(LO)) | diff) & HI;
    (nonzero >> 7).count_ones() as u64
}

fn levenshtein_dist_branchless_reference(val: u64, aux: u64) -> u64 {
    let vb = val.to_le_bytes();
    let ab = aux.to_le_bytes();
    let mut count = 0u64;
    for i in 0..8 {
        if vb[i] != ab[i] {
            count += 1;
        }
    }
    count
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
        if levenshtein_dist_branchless(v, a) != levenshtein_dist_branchless_reference(v, a) {
            println!("BUG FOUND in levenshtein: v={}, a={}", v, a);
            return;
        }
    }
    println!("levenshtein OK");
}
