use bcinr_cmca::sa2a::*;
#[test] fn sa2a_interop(){ assert!(interop::allocate(&Envelope::powerless("s","e","r"),&ResourceEnvelope{cpu:2,memory:2,io:2},&Allocation{cpu:1,memory:1,io:1}).is_ok()); }
