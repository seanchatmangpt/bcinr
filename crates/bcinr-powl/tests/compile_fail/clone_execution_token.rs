// Must not compile: ExecutionToken is deliberately non-Clone.
//
// We obtain a token via the legitimate pipeline so this test does not depend
// on the `testing` feature or the gated `new_for_test` constructor.
use bcinr_powl::tape::v2::{OpKind, Powl64Op, PowlTape};
use bcinr_powl::typestate::{PowlRunner, TopologyKind};

fn main() {
    let mut tape = PowlTape::new();
    let mut op0 = Powl64Op::silent();
    op0.op_kind = OpKind::Activity;
    tape.push(op0).unwrap();
    tape.entry_op = 0;
    tape.exit_op = 0;

    let runner = PowlRunner::new(tape);
    let compiled = runner.validate().unwrap();
    let scheduled = compiled.schedule::<{ TopologyKind::Standard }>();
    let (_exec, tok) = scheduled.begin_execution();
    let _tok2 = tok.clone(); // ERROR: no method `clone` — ExecutionToken is non-Clone
}
