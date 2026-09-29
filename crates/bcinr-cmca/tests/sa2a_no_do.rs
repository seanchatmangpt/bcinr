use bcinr_cmca::sa2a::*;
#[test] fn sa2a_no_do(){ assert_eq!(Envelope::powerless("subject","effect","replay").authority,"none"); }
