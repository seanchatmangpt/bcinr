use bcinr_cmca::sa2a::*;
#[test]
fn sa2a_budget_exact() {
    assert!(ResourceEnvelope {
        cpu: 1,
        memory: 2,
        io: 3
    }
    .admits(&Allocation {
        cpu: 1,
        memory: 2,
        io: 3
    }));
}
