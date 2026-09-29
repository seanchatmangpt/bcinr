use bcinr_cmca::sa2a::*;
#[test] fn sa2a_recovery_refused(){ assert_eq!(recovery::decide(&Receipt{effect_id:"e",replay_id:"r",outcome:Outcome::Refused}),RecoveryDecision::Replan); }
