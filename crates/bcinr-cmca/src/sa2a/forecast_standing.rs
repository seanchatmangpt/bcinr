//! Forecast verification, calibration, and shock standing.
//!
//! Standing is evidence. It can support later admission decisions but cannot
//! grant allocation or execution authority.

use super::predictive_model::{ForecastMatrix, ResourceVector};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ForecastStanding {
    pub sample_count: u64,
    pub mean_squared_error: u64,
    pub mean_absolute_error: u64,
    pub root_mean_squared_error: u64,
    pub signed_bias: i64,
    pub interval_coverage_ppm: u32,
    pub shock_misses: u64,
    pub shock_false_positives: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ForecastCalibrationReceipt {
    pub subject_id: u64,
    pub observation_digest: u64,
    pub model_digest: u64,
    pub provenance_digest: u64,
    pub standing: ForecastStanding,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShockPolicy<const R: usize> {
    pub thresholds: ResourceVector<R>,
}

pub fn evaluate_forecast<const R: usize, const H: usize>(
    matrix: &ForecastMatrix<R, H>,
    actual: &[ResourceVector<R>; H],
    baseline: ResourceVector<R>,
    shock_policy: ShockPolicy<R>,
) -> ForecastCalibrationReceipt {
    let mut absolute_sum = 0u128;
    let mut squared_sum = 0u128;
    let mut signed_sum = 0i128;
    let mut interval_hits = 0u64;
    let mut shock_misses = 0u64;
    let mut shock_false_positives = 0u64;
    let mut samples = 0u64;

    let mut axis = 0usize;
    while axis < R {
        let mut horizon = 0usize;
        while horizon < H {
            let cell = matrix.cells[axis][horizon];
            let observed = actual[horizon].values[axis];
            let error = observed.abs_diff(cell.expected);

            absolute_sum = absolute_sum.saturating_add(error as u128);
            squared_sum = squared_sum.saturating_add((error as u128).saturating_mul(error as u128));
            signed_sum = signed_sum.saturating_add((observed as i128) - (cell.expected as i128));

            if observed >= cell.lower && observed <= cell.upper {
                interval_hits = interval_hits.saturating_add(1);
            }

            let actual_shock =
                observed.abs_diff(baseline.values[axis]) > shock_policy.thresholds.values[axis];
            let predicted_shock = cell.expected.abs_diff(baseline.values[axis])
                > shock_policy.thresholds.values[axis];

            if actual_shock && !predicted_shock {
                shock_misses = shock_misses.saturating_add(1);
            }
            if predicted_shock && !actual_shock {
                shock_false_positives = shock_false_positives.saturating_add(1);
            }

            samples = samples.saturating_add(1);
            horizon += 1;
        }
        axis += 1;
    }

    let divisor = core::cmp::max(samples, 1) as u128;
    let mse_u128 = squared_sum / divisor;
    let mae_u128 = absolute_sum / divisor;
    let coverage = ((interval_hits as u128) * 1_000_000u128) / divisor;
    let bias_i128 = signed_sum / (divisor as i128);

    let mean_squared_error = saturating_u128_to_u64(mse_u128);
    let mean_absolute_error = saturating_u128_to_u64(mae_u128);
    let signed_bias = saturating_i128_to_i64(bias_i128);

    ForecastCalibrationReceipt {
        subject_id: matrix.subject_id,
        observation_digest: matrix.observation_digest,
        model_digest: matrix.model_digest,
        provenance_digest: matrix.provenance_digest,
        standing: ForecastStanding {
            sample_count: samples,
            mean_squared_error,
            mean_absolute_error,
            root_mean_squared_error: integer_sqrt(mean_squared_error),
            signed_bias,
            interval_coverage_ppm: core::cmp::min(coverage, 1_000_000) as u32,
            shock_misses,
            shock_false_positives,
        },
    }
}

/// Deterministic standing digest suitable for binding a later powerless adapter.
pub fn standing_digest(standing: &ForecastStanding) -> u64 {
    let mut digest = 0xA5A5_5A5A_1357_2468u64;
    digest = mix64(digest, standing.sample_count);
    digest = mix64(digest, standing.mean_squared_error);
    digest = mix64(digest, standing.mean_absolute_error);
    digest = mix64(digest, standing.root_mean_squared_error);
    digest = mix64(digest, standing.signed_bias as u64);
    digest = mix64(digest, standing.interval_coverage_ppm as u64);
    digest = mix64(digest, standing.shock_misses);
    mix64(digest, standing.shock_false_positives)
}

fn integer_sqrt(value: u64) -> u64 {
    if value < 2 {
        return value;
    }

    let mut low = 1u64;
    let mut high = core::cmp::min(value, u32::MAX as u64 + 1);
    let mut answer = 1u64;

    while low <= high {
        let mid = low + ((high - low) / 2);
        let square = (mid as u128) * (mid as u128);
        if square <= value as u128 {
            answer = mid;
            low = mid.saturating_add(1);
        } else {
            high = mid.saturating_sub(1);
        }
    }

    answer
}

fn saturating_u128_to_u64(value: u128) -> u64 {
    core::cmp::min(value, u64::MAX as u128) as u64
}

fn saturating_i128_to_i64(value: i128) -> i64 {
    if value > i64::MAX as i128 {
        i64::MAX
    } else if value < i64::MIN as i128 {
        i64::MIN
    } else {
        value as i64
    }
}

#[inline(always)]
fn mix64(a: u64, b: u64) -> u64 {
    let mut x = a ^ b.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    x
}
