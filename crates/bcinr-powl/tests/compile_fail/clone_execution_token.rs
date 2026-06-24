// Must not compile: ExecutionToken is deliberately non-Clone.
fn main() {
    let tok = bcinr_powl::typestate::ExecutionToken::new_for_test(0b11, 2);
    let _tok2 = tok.clone();
}
