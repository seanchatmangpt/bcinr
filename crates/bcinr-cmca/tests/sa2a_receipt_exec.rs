use bcinr_cmca::sa2a::receipt::{Outcome,Receipt};
use bcinr_cmca::sa2a::*;
#[test] fn sa2a_receipt_exec(){ assert_eq!(Receipt{effect_id:"e",replay_id:"r",outcome:Outcome::Executed}.recovery(),RecoveryDecision::Terminal); }
