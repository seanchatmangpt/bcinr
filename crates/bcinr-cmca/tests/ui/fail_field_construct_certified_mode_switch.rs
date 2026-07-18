use bcinr_cmca::mode_switch::CertifiedModeSwitch;
fn main() {
    let _ = CertifiedModeSwitch {
        admitted_state_digest: 0,
        target_mode_digest: 0,
        prepared_digest: 0,
    };
}
