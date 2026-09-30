use bcinr_cmca::sa2a::*;
#[test] fn sa2a_replay(){ assert!(replay::ReplayKey{subject:"s",effect_id:"e",replay_id:"r"}.same_attempt(&replay::ReplayKey{subject:"s",effect_id:"e",replay_id:"r"})); }
