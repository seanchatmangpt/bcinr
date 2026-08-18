//! Cross-repository contract evidence for the BCINR CMCA -> MFW mass bridge.
//!
//! BCINR retains allocation/certification authority. This test only proves the
//! fixed-width data boundary consumed by MFW: eight non-negative Q16.16 lanes
//! whose raw bits can be quantized into MFW's bounded `u8` candidate mass.
//!
//! The exact consumer subject is the open MFW CMCA bridge candidate. Binding
//! that identity here makes drift explicit without importing MFW authority or
//! giving this producer-side court ambient actuation rights.

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

/// Frozen BCINR producer base inspected for this v26.9.1 seam closure.
const BCINR_PRODUCER_BASE: &str = "d6fefefdb95df5dbbb520afe7b5a4df53aa6e27f";
/// Exact MFW consumer candidate from draft PR #66.
const MFW_CONSUMER_HEAD: &str = "a808821c6636535bc80f59f660b5b35906948c8e";
/// MFW's public bridge constant for unsigned Q16.16 `1.0`.
const MFW_Q16_ONE: u32 = 65_536;

const _: [(); 8] = [(); N];

fn q16_16_to_u8_mass(bits: u32) -> u8 {
    (((bits as u64) * 255 + 32_768) >> 16) as u8
}

#[test]
fn exact_cross_repo_subjects_are_frozen_and_non_genesis() {
    for (name, sha) in [
        ("bcinr_producer_base", BCINR_PRODUCER_BASE),
        ("mfw_consumer_head", MFW_CONSUMER_HEAD),
    ] {
        assert_eq!(sha.len(), 40, "{name} must be an exact SHA-1 identity");
        assert!(
            sha.bytes().all(|byte| byte.is_ascii_hexdigit()),
            "{name} must be hexadecimal"
        );
        assert_ne!(sha, "0000000000000000000000000000000000000000");
    }
    assert_ne!(BCINR_PRODUCER_BASE, MFW_CONSUMER_HEAD);
}

#[test]
fn allocator_output_is_an_eight_lane_q16_16_mass_vector() {
    let parent = [-1; N];
    let mut weights = [[NonNegativeFixed::ONE; 2 * Q]; N];
    let payoffs = [[NonNegativeFixed::ZERO; 2 * Q]; N];
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
        &payoffs,
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
    assert_eq!(one, MFW_Q16_ONE, "the bridge contract is explicitly Q16.16");
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
}

#[test]
fn mfw_quantization_contract_is_bounded_monotone_and_endpoint_exact() {
    assert_eq!(q16_16_to_u8_mass(0), 0);
    assert_eq!(q16_16_to_u8_mass(MFW_Q16_ONE), 255);

    let mut previous = q16_16_to_u8_mass(0);
    for bits in 1..=MFW_Q16_ONE {
        let current = q16_16_to_u8_mass(bits);
        assert!(
            current >= previous,
            "quantization must be monotone at Q16.16 value {bits}"
        );
        previous = current;
    }
}
