use super::{
    contract::RecoveryDecision,
    receipt::{Outcome, Receipt},
};
pub const fn decide(r: &Receipt<'_>) -> RecoveryDecision {
    match r.outcome {
        Outcome::Unknown => RecoveryDecision::Reconcile,
        Outcome::Refused => RecoveryDecision::Replan,
        Outcome::Executed => RecoveryDecision::Terminal,
    }
}
