use bcinr_cmca::mode_switch::{ActuationEvidence, ActuationOutcome};
fn main() {
    let _ = ActuationEvidence {
        certificate_digest: 0,
        old_control_mode_digest: 0,
        new_control_mode_digest: 0,
        round_identity: 0,
        outcome: ActuationOutcome::Applied,
    };
}
