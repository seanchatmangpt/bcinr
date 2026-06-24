//! Branchless selection primitives (HAND-AUTHORED).
//!
//! Stub for the branchless argmax used by move selection. CC = 1.

/// All-ones mask if `x != 0`, else all-zeros. Branchless.
#[inline(always)]
#[must_use]
pub fn nz_mask(x: u64) -> u64 {
    (((x | x.wrapping_neg()) >> 63) & 1).wrapping_neg()
}

/// Branchless select: `b` where mask bits are set, else `a`.
#[inline(always)]
#[must_use]
pub fn select_u64(mask: u64, a: u64, b: u64) -> u64 {
    a ^ (mask & (a ^ b))
}

/// Branchless argmax stub: returns `cand_idx` when `cand > best`, else
/// `best_idx`. Uses sign-mask arithmetic, no branches.
#[inline(always)]
#[must_use]
pub fn argmax_step(best: i32, best_idx: u32, cand: i32, cand_idx: u32) -> (i32, u32) {
    let gt = (((best as i64) - (cand as i64)) >> 63) as i32; // -1 if cand>best else 0
    let mask = gt as u32;
    let new_best = best ^ (gt & (best ^ cand));
    let new_idx = best_idx ^ (mask & (best_idx ^ cand_idx));
    (new_best, new_idx)
}

/// Branchless argmax over a slice of candidate scores.
///
/// Returns the index of the maximum score (lowest index on ties). The per-step
/// update is branchless sign-mask arithmetic ([`argmax_step`]); the reduction
/// iterates the candidate set (this is the selection boundary, not a CC=1
/// station kernel). Returns `0` for an empty slice.
#[must_use]
pub fn argmax_i32(scores: &[i32]) -> u32 {
    let mut best = i32::MIN;
    let mut best_idx: u32 = 0;
    let mut i: u32 = 0;
    while (i as usize) < scores.len() {
        let (nb, ni) = argmax_step(best, best_idx, scores[i as usize], i);
        best = nb;
        best_idx = ni;
        i += 1;
    }
    best_idx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn argmax_picks_max() {
        assert_eq!(argmax_i32(&[1, 9, 3, 9, 2]), 1, "lowest index on ties");
        assert_eq!(argmax_i32(&[-5, -2, -9]), 1);
        assert_eq!(argmax_i32(&[42]), 0);
    }
}
