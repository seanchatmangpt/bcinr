//! Heterogeneous forecast benchmark and counterfactual replay receipts.

use super::forecast_standing::{evaluate_forecast, ForecastCalibrationReceipt, ShockPolicy};
use super::predictive_model::{ForecastMatrix, ResourceVector};
use super::resource_envelope::Allocation;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CorpusClass {
    Cluster,
    Web,
    Hpc,
    Synthetic,
    Adversarial,
    Replay,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BenchmarkScenario<const R: usize, const H: usize> {
    pub class: CorpusClass,
    pub corpus_digest: u64,
    pub baseline: ResourceVector<R>,
    pub actual: [ResourceVector<R>; H],
    pub shock_policy: ShockPolicy<R>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PredictorCostReceipt {
    pub training_time_micros: u64,
    pub inference_time_micros: u64,
    pub peak_memory_bytes: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BenchmarkReceipt {
    pub class: CorpusClass,
    pub corpus_digest: u64,
    pub calibration: ForecastCalibrationReceipt,
    pub cost: PredictorCostReceipt,
}

pub fn benchmark_forecast<const R: usize, const H: usize>(
    matrix: &ForecastMatrix<R, H>,
    scenario: &BenchmarkScenario<R, H>,
    cost: PredictorCostReceipt,
) -> BenchmarkReceipt {
    BenchmarkReceipt {
        class: scenario.class,
        corpus_digest: scenario.corpus_digest,
        calibration: evaluate_forecast(
            matrix,
            &scenario.actual,
            scenario.baseline,
            scenario.shock_policy,
        ),
        cost,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CounterfactualReplayReceipt {
    pub subject_id: u64,
    pub observation_digest: u64,
    pub forecast_digest: u64,
    pub demand: Allocation,
    pub reactive: Allocation,
    pub predictive: Allocation,
    pub reactive_unmet: Allocation,
    pub predictive_unmet: Allocation,
    pub reactive_waste: Allocation,
    pub predictive_waste: Allocation,
    pub allocation_changed: bool,
}

/// Compare two already-computed allocation decisions against the same observed
/// demand. This function never calls an allocator and never actuates either
/// allocation.
pub fn counterfactual_replay(
    subject_id: u64,
    observation_digest: u64,
    forecast_digest: u64,
    demand: Allocation,
    reactive: Allocation,
    predictive: Allocation,
) -> CounterfactualReplayReceipt {
    CounterfactualReplayReceipt {
        subject_id,
        observation_digest,
        forecast_digest,
        demand,
        reactive,
        predictive,
        reactive_unmet: unmet(demand, reactive),
        predictive_unmet: unmet(demand, predictive),
        reactive_waste: waste(demand, reactive),
        predictive_waste: waste(demand, predictive),
        allocation_changed: reactive != predictive,
    }
}

const fn unmet(demand: Allocation, allocation: Allocation) -> Allocation {
    Allocation {
        cpu: demand.cpu.saturating_sub(allocation.cpu),
        memory: demand.memory.saturating_sub(allocation.memory),
        io: demand.io.saturating_sub(allocation.io),
    }
}

const fn waste(demand: Allocation, allocation: Allocation) -> Allocation {
    Allocation {
        cpu: allocation.cpu.saturating_sub(demand.cpu),
        memory: allocation.memory.saturating_sub(demand.memory),
        io: allocation.io.saturating_sub(demand.io),
    }
}
