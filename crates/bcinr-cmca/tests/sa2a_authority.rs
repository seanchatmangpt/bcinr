use bcinr_cmca::sa2a::*;
#[test] fn sa2a_authority(){ assert_eq!(Envelope::powerless("s","e","r").authority,"none"); }
