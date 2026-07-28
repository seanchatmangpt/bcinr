//! Public analysis-cascade boundary aligned to the admitted CMCA lens domain.

pub use crate::cascade_legacy::{admit_fixed, CascadeRefusal, CascadeTree, NumericContext};

use alloc::vec::Vec;
use crate::fixed::NonNegativeFixed;

/// Largest admitted integer lens magnitude. The analysis and certified rails
/// share the mathematical domain `q ∈ [-2, 2]`; a larger repeated-loop bound is
/// not permission to widen that domain.
pub const MAX_LENS_MAGNITUDE: u32 = crate::generated_profile::MAX_LENS_MAGNITUDE;

pub fn escort_weight(
    mass: NonNegativeFixed,
    lens: i32,
    node: usize,
) -> Result<NonNegativeFixed, CascadeRefusal> {
    admit_lens(lens)?;
    crate::cascade_legacy::escort_weight(mass, lens, node)
}

pub fn consequence_mass(
    tree: &CascadeTree,
    lenses: &[i32],
) -> Result<Vec<NonNegativeFixed>, CascadeRefusal> {
    for &lens in lenses {
        admit_lens(lens)?;
    }
    crate::cascade_legacy::consequence_mass(tree, lenses)
}

fn admit_lens(lens: i32) -> Result<(), CascadeRefusal> {
    if lens.unsigned_abs() > MAX_LENS_MAGNITUDE {
        Err(CascadeRefusal::ExponentOutOfRange {
            lens,
            max_magnitude: MAX_LENS_MAGNITUDE,
        })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analysis_rail_does_not_widen_certified_lens_domain() {
        assert!(matches!(
            escort_weight(NonNegativeFixed::ONE, 3, 0),
            Err(CascadeRefusal::ExponentOutOfRange {
                lens: 3,
                max_magnitude: 2
            })
        ));
    }
}
