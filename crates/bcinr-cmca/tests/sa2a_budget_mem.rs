use bcinr_cmca::sa2a::*;
#[test]
fn sa2a_budget_mem() {
    assert!(!ResourceEnvelope {
        cpu: 1,
        memory: 2,
        io: 3
    }
    .admits(&Allocation {
        cpu: 1,
        memory: 3,
        io: 3
    }));
}
