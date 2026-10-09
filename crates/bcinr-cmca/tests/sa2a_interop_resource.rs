use bcinr_cmca::sa2a::*;
#[test]
fn sa2a_interop_resource() {
    assert_eq!(
        interop::allocate_via_envelope(
            &Envelope::powerless("s", "e", "r"),
            &ResourceEnvelope {
                cpu: 0,
                memory: 0,
                io: 0
            },
            &Allocation {
                cpu: 1,
                memory: 0,
                io: 0
            }
        ),
        Err(interop::InteropRefusal::Resource)
    );
}
