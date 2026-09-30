use bcinr_cmca::sa2a::*;
#[test] fn sa2a_ocel(){ assert_eq!(ocel::OcelEvent::receipt("s","e","r").event_type,"sa2a.receipt"); }
