use bcinr_cmca::sa2a::*;
#[test]
fn sa2a_empty_subject() {
    assert_eq!(
        admit(&Envelope::powerless("", "e", "r")),
        Err(Refusal::EmptySubject)
    );
}
