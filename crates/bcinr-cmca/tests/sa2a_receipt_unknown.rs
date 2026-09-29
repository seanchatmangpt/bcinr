use bcinr_cmca::sa2a::*;
#[test] fn sa2a_receipt_unknown(){ assert_eq!(Receipt{effect_id:"e",replay_id:"r",outcome:Outcome::Unknown}.recovery(),RecoveryDecision::Reconcile); }
