use bcinr_cmca::fixed::SignedFixed;
use bcinr_cmca::observatory::ObservatoryFlagSet;
use bcinr_cmca::proposal::ModeProposal;
fn main() {
    let _ = ModeProposal {
        proposed_control_delta: SignedFixed::ZERO,
        observation_digest: 0,
        current_mode_digest: 0,
        round_identity: 0,
        flags: ObservatoryFlagSet::EMPTY,
        proposal_digest: 0,
    };
}
