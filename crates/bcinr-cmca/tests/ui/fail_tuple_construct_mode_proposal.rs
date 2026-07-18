use bcinr_cmca::fixed::SignedFixed;
use bcinr_cmca::observatory::ObservatoryFlagSet;
use bcinr_cmca::proposal::ModeProposal;
fn main() {
    let _ = ModeProposal(SignedFixed::ZERO, 0u64, 0u64, 0u64, ObservatoryFlagSet::EMPTY, 0u64);
}
