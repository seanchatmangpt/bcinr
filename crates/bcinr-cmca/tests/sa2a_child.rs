use bcinr_cmca::sa2a::*;
#[test]
fn sa2a_child() {
    assert_eq!(
        ResourceEnvelope {
            cpu: 3,
            memory: 3,
            io: 3
        }
        .child(&Allocation {
            cpu: 1,
            memory: 1,
            io: 1
        }),
        Some(ResourceEnvelope {
            cpu: 2,
            memory: 2,
            io: 2
        })
    );
}
