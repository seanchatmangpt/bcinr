use bcinr_cmca::sa2a::*;
#[test] fn sa2a_empty_effect(){ assert_eq!(admit(&Envelope::powerless("s","","r")),Err(Refusal::EmptyEffect)); }
