// GENERATED STABILITY PROFILE
// Expanded Stochastic Homeostasis Envelope

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Fixed {
    pub raw: u64,
}

impl Fixed {
    pub const fn from_raw(raw: u64) -> Self {
        Self { raw }
    }
}

pub type Digest = [u8; 32];

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct StabilityProfile {
    pub gain_matrix: [[Fixed; 5]; 5],
    pub weight_vector: [Fixed; 5],
    pub deterministic_margin: Fixed,

    pub noise_second_moment_bounds: [Fixed; 5],
    pub certified_noise_radius: Fixed,

    pub mode_jump_bound: Fixed,
    pub minimum_dwell_rounds: u32,
    pub certified_switching_radius: Fixed,

    pub total_homeostatic_radius: Fixed,

    pub temperature_ceiling: Fixed,
    pub distinguishability_floor: Fixed,
    pub floor_minimum: Fixed,

    pub certificate_digest: Digest,
}

pub const PROFILE: StabilityProfile = StabilityProfile {
    gain_matrix: [
        [Fixed::from_raw(800_000_000), Fixed::from_raw(10_000_000), Fixed::from_raw(5_000_000), Fixed::from_raw(2_000_000), Fixed::from_raw(1_000_000)],
        [Fixed::from_raw(10_000_000), Fixed::from_raw(850_000_000), Fixed::from_raw(8_000_000), Fixed::from_raw(3_000_000), Fixed::from_raw(2_000_000)],
        [Fixed::from_raw(5_000_000), Fixed::from_raw(8_000_000), Fixed::from_raw(900_000_000), Fixed::from_raw(4_000_000), Fixed::from_raw(1_000_000)],
        [Fixed::from_raw(1_000_000), Fixed::from_raw(2_000_000), Fixed::from_raw(3_000_000), Fixed::from_raw(950_000_000), Fixed::from_raw(1_000_000)],
        [Fixed::from_raw(1_000_000), Fixed::from_raw(1_000_000), Fixed::from_raw(1_000_000), Fixed::from_raw(1_000_000), Fixed::from_raw(980_000_000)],
    ],
    weight_vector: [
        Fixed::from_raw(1_000_000_000),
        Fixed::from_raw(1_200_000_000),
        Fixed::from_raw(1_500_000_000),
        Fixed::from_raw(800_000_000),
        Fixed::from_raw(500_000_000),
    ],
    deterministic_margin: Fixed::from_raw(10_000_000),

    noise_second_moment_bounds: [
        Fixed::from_raw(1_000_000),
        Fixed::from_raw(1_200_000),
        Fixed::from_raw(800_000),
        Fixed::from_raw(2_500_000),
        Fixed::from_raw(3_000_000),
    ],
    certified_noise_radius: Fixed::from_raw(45_000_000),

    mode_jump_bound: Fixed::from_raw(200_000_000),
    minimum_dwell_rounds: 461,
    certified_switching_radius: Fixed::from_raw(75_000_000),

    total_homeostatic_radius: Fixed::from_raw(120_000_000),

    temperature_ceiling: Fixed::from_raw(5_000_000_000),
    distinguishability_floor: Fixed::from_raw(1_000_000),
    floor_minimum: Fixed::from_raw(5_000_000),

    certificate_digest: [
        0x2c, 0xf2, 0x4d, 0xba, 0x5f, 0xb0, 0xa3, 0x0e,
        0x26, 0xe8, 0x3b, 0x2a, 0xc5, 0xb9, 0xe2, 0x9e,
        0x1b, 0x16, 0x1e, 0x5c, 0x1f, 0xa7, 0x42, 0x5e,
        0x73, 0x04, 0x33, 0x62, 0x93, 0x8b, 0x98, 0x24
    ],
};

// Aliases for compatibility with the allocator implementation:
pub const GAIN_MATRIX: [[Fixed; 5]; 5] = PROFILE.gain_matrix;
pub const WEIGHT_VECTOR: [Fixed; 5] = PROFILE.weight_vector;
pub const CONTRACTION_MARGIN: Fixed = PROFILE.deterministic_margin;
pub const CERTIFICATE_DIGEST: Digest = PROFILE.certificate_digest;
pub const MODE_DWELL_ROUNDS_MIN: u32 = PROFILE.minimum_dwell_rounds;

pub const BETA_M_MAX: Fixed = PROFILE.certified_noise_radius; // 45_000_000
pub const ZETA_W_MAX: Fixed = Fixed::from_raw(12_500_000); // 0.0125
pub const ETA_G_MIN: Fixed = PROFILE.distinguishability_floor; // 1_000_000
