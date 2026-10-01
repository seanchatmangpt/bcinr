use bcinr_cmca::sa2a::*;
#[test] fn sa2a_empty_replay(){ assert_eq!(admit(&Envelope::powerless("s","e","")),Err(Refusal::EmptyReplay)); }
