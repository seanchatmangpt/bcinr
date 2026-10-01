use bcinr_cmca::sa2a::*;
#[test] fn sa2a_admit(){ assert!(admit(&Envelope::powerless("s","e","r")).is_ok()); }
