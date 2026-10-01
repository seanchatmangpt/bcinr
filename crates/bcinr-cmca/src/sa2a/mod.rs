//! SA2A third-consumer boundary for CMCA.
//! This module consumes powerless PreparedEffect/resource/receipt identities only.
//! It never mints authority or performs effects.
pub mod admission;
pub mod contract;
pub mod forecast_benchmark;
pub mod forecast_standing;
pub mod interop;
pub mod ocel;
pub mod prepared_effect_ref;
pub mod predictive_envelope;
pub mod predictive_measure;
pub mod predictive_model;
pub mod receipt;
pub mod recovery;
pub mod reservation;
pub mod replay;
pub mod resource_envelope;
pub mod shadow_optimizer;
pub mod symbolic_envelope;

pub use admission::{admit, Refusal};
pub use contract::{Envelope, RecoveryDecision, CONTRACT};
pub use forecast_benchmark::{
    benchmark_forecast, counterfactual_replay, BenchmarkReceipt, BenchmarkScenario, CorpusClass,
    CounterfactualReplayReceipt, PredictorCostReceipt,
};
pub use forecast_standing::{
    evaluate_forecast, standing_digest, ForecastCalibrationReceipt, ForecastStanding, ShockPolicy,
};
pub use predictive_envelope::{
    admit_predictive_envelope, AdmittedPredictiveConsequenceEnvelope, ForecastReceipt,
    PredictiveAdmission, PredictiveConsequenceEnvelope, PredictivePressure, PredictiveRefusal,
    PressureVector, ResourceAxis, PRESSURE_PPM_ONE,
};
pub use predictive_measure::{
    admit_predictive_measure, build_predictive_measure, predictive_envelope_digest,
    AdmittedPredictiveMeasure, PredictiveMeasureAdmission, PredictiveMeasureArtifact,
    PredictiveMeasureRefusal,
};
pub use predictive_model::{
    admit_forecast_matrix, ForecastCell, ForecastHorizon, ForecastHorizonSet, ForecastMatrix,
    ForecastModelRefusal, Predictor, PredictorArtifact, ResourceAxisId, ResourceAxisRegistry,
    ResourceVector, TraceSample, TraceWindow, CONFIDENCE_PPM_ONE,
};
pub use resource_envelope::{Allocation, BudgetLedger, BudgetReceipt, ResourceEnvelope};
pub use reservation::{
    admit_reservation_proposal, AdmittedReservationProposal, ReservationAdmission,
    ReservationProposal, ReservationRefusal,
};
pub use shadow_optimizer::{
    analyze_population, attraction_proposal, optimization_receipt, shadow_rejuvenation_mask,
    OptimizationReceipt, PopulationAnalysis, ShadowCandidate, ShadowOptimizerRefusal,
};
pub use symbolic_envelope::{
    peak_reusable_usage, pipeline_latency, AffineResourceExpr, PipelineKnobs, SymbolDomain,
    SymbolicFamily, SymbolicRefusal, SymbolicWitness, TimedAllocation,
};
