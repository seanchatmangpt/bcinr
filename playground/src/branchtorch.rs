#![allow(warnings, clippy::all)]
#![allow(warnings)]
//! BranchTorch: Zero-Allocation Branchless Training Framework.
//!
//! A `#![no_std]` evolutionary matrix that strictly mutates and trains
//! Binarized Graph Neural Networks (BGNN) using CC=1 polynomial physics,
//! completely bypassing external frameworks like PyTorch.

use crate::gnn::BinarizedGnnLayer;

/// A perfectly deterministic, branchless Xorshift pseudo-random number generator.
/// Used to trigger mutation topologies across neural weights.
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct BranchlessRng {
    /// Current generator state.
    pub seed: u64,
    /// Padding to reach the 64-byte cache-line-aligned layout.
    pub _pad: [u8; 56],
}

impl BranchlessRng {
    /// Generates the next random u64 using pure branchless arithmetic.
    #[inline(always)]
    pub fn next(&mut self) -> u64 {
        let mut x = self.seed;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.seed = x;
        x
    }
}

/// Executes a branchless evolutionary mutation step across a BGNN layer.
///
/// Instead of backpropagation (which relies on expensive floating-point gradients
/// and calculus), BranchTorch utilizes bitwise Genetic Algorithm permutations.
/// It iterates over the neural matrix and stochastically flips weight bits
/// based on the RNG mask in constant O(1) clock cycles.
///
/// # Example
/// ```
/// use playground::gnn::BinarizedGnnLayer;
/// use playground::branchtorch::{BranchlessRng, mutate_weights_branchless};
///
/// let mut layer = BinarizedGnnLayer { weights: [0xFFFF_0000_FFFF_0000; 64], bias: 0 };
/// let mut rng = BranchlessRng { seed: 0xDEADBEEF_CAFEBABE, _pad: [0; 56] };
///
/// // The original first weight mask
/// let original = layer.weights[0];
///
/// // Execute 1 generation of branchless evolutionary mutation
/// mutate_weights_branchless(&mut layer, &mut rng).unwrap();
///
/// // The matrix has physically mutated its internal weights!
/// assert_ne!(layer.weights[0], original);
/// ```
#[inline(always)]
pub fn mutate_weights_branchless(
    layer: &mut BinarizedGnnLayer,
    rng: &mut BranchlessRng,
) -> Result<(), &'static str> {
    // CC=1 unrolled mutation matrix
    for i in 0..64 {
        // Generate a random bitmask
        let mutation_mask = rng.next();

        // Isolate roughly ~1.5% of bits to randomly flip using sparse ANDing
        let sparse_flip = mutation_mask & rng.next() & rng.next() & rng.next();

        // Branchlessly flip the selected weights using XOR
        layer.weights[i] ^= sparse_flip;
    }

    // Mutate bias
    layer.bias ^= rng.next() & rng.next() & rng.next() & rng.next();

    Ok(())
}
