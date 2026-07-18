use bcinr_cmca::proposal::{AdmittedProposal, ModeProposal};
fn main() {
    let proposal: ModeProposal = unreachable!();
    let _ = AdmittedProposal { proposal };
}
