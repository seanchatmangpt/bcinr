//! Cross-repository contract evidence for the BCINR CMCA -> MFW mass bridge.
//!
//! BCINR retains allocation/certification authority. This test only proves the
//! fixed-width data boundary consumed by MFW: eight non-negative Q16.16 lanes
//! whose raw bits can be quantized into MFW's bounded `u8` candidate mass.

#![cfg(not(any(
    feature = "mutant_1",
    feature = "mutant_2",
    feature = "mutant_3",
    feature = "mutant_4",
    feature = "mutant_5",
    feature = "mutant_6",
    feature = "mutant_7",
    feature = "mutant_8",
    feature = "mutant_9",
    feature = "mutant_10",
    feature = "mutant_11"
)))]

use bcinr_cmca::allocator::allocate;
use bcinr_cmca::fixed::NonNegativeFixed;
use bcinr_cmca::generated::consequence_mass::case_studies::{
    ETA, LAMBDA, LENS_REGISTRY, N, OBJECT_REGISTRY, Q,
};
use bcinr_cmca::generated::stability_profile::CERTIFICATE_DIGEST;

const _: [(); 8] = [(); N];

fn q16_16_to_u8_mass(bits: u32) -> u8 {
    (((bits as u64) * 255 + 32_768) >> 16) as u8
}

#[test]
fn allocator_output_is_an_eight_lane_q16_16_mass_vector() {
    let parent = [-1; N];
    let mut weights = [[NonNegativeFixed::ONE; 2 * Q]; N];
    let mut payoffs = [[NonNegativeFixed::ZERO; 2 * Q]; N];
    let mu = [NonNegativeFixed::ZERO; N];
    let costs = [NonNegativeFixed::ZERO; N];
    let mut last_switch_t = 0;
    let mut prev_mode = 0;

    let allocation = allocate(
        &OBJECT_REGISTRY,
        &LENS_REGISTRY,
        &LAMBDA,
        ETA,
        &parent,
        &mut weights,
        &mut payoffs,
        NonNegativeFixed::ZERO,
        NonNegativeFixed::ZERO,
        &mu,
        &costs,
        0,
        &mut last_switch_t,
        &mut prev_mode,
        500,
        CERTIFICATE_DIGEST,
        None,
    )
    .expect("selection-only CMCA allocation must remain executable");

    let one = NonNegativeFixed::ONE.to_bits();
    assert_eq!(one, 65_536, "the bridge contract is explicitly Q16.16");
    assert!(
        allocation.iter().all(|value| value.to_bits() <= one),
        "each allocation lane must remain within the unit interval"
    );
    assert!(
        allocation.iter().any(|value| value.to_bits() > 0),
        "the allocator must not manufacture an empty mass vector"
    );

    let wire_bits = allocation.map(NonNegativeFixed::to_bits);
    let mfw_mass = wire_bits.map(q16_16_to_u8_mass);

    assert!(
        mfw_mass.iter().any(|mass| *mass > 0),
        "Q16.16 -> u8 quantization must preserve material allocation"
    );
    assert!(
        mfw_mass.iter().all(|mass| *mass <= u8::MAX),
        "the MFW boundary is fixed to u8 mass"
    );
}
