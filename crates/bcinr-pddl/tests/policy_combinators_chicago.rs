//! Chicago TDD coverage for deterministic policy composition.

#![cfg(feature = "mfw-planner")]

use bcinr_pddl::prelude::*;

#[derive(Debug, Clone, Copy)]
struct Admit(&'static str);

impl PolicyIdentity for Admit {
    fn root(&self) -> PolicySetRoot {
        PolicySetRoot::hash_parts(&[b"admit", self.0.as_bytes()])
    }
}

impl Policy<(), ()> for Admit {
    type Evidence = &'static str;
    type Refusal = &'static str;

    fn evaluate(&self, _input: &(), _context: &()) -> PolicyDecision<Self::Evidence, Self::Refusal> {
        PolicyDecision::Admit(self.0)
    }
}

#[derive(Debug, Clone, Copy)]
struct Refuse(&'static str);

impl PolicyIdentity for Refuse {
    fn root(&self) -> PolicySetRoot {
        PolicySetRoot::hash_parts(&[b"refuse", self.0.as_bytes()])
    }
}

impl Policy<(), ()> for Refuse {
    type Evidence = &'static str;
    type Refusal = &'static str;

    fn evaluate(&self, _input: &(), _context: &()) -> PolicyDecision<Self::Evidence, Self::Refusal> {
        PolicyDecision::Refuse(self.0)
    }
}

chicago_tdd_tools::test!(any_policy_uses_second_only_after_first_refuses, {
    let policy = AnyPolicy::new(Refuse("first-refused"), Admit("second-admitted"));
    assert_eq!(
        policy.evaluate(&(), &()),
        PolicyDecision::Admit(AnyPolicyEvidence::Second("second-admitted"))
    );
});

chicago_tdd_tools::test!(any_policy_preserves_both_refusals, {
    let policy = AnyPolicy::new(Refuse("first-refused"), Refuse("second-refused"));
    assert_eq!(
        policy.evaluate(&(), &()),
        PolicyDecision::Refuse(AnyPolicyRefusal {
            first: "first-refused",
            second: "second-refused",
        })
    );
});

chicago_tdd_tools::test!(not_policy_exchanges_evidence_and_refusal, {
    assert_eq!(
        NotPolicy::new(Admit("was-admitted")).evaluate(&(), &()),
        PolicyDecision::Refuse("was-admitted")
    );
    assert_eq!(
        NotPolicy::new(Refuse("was-refused")).evaluate(&(), &()),
        PolicyDecision::Admit("was-refused")
    );
});

chicago_tdd_tools::test!(policy_operator_identity_is_not_ambiguous, {
    let left = Admit("left");
    let right = Admit("right");
    let all = AllPolicy::new(left, right);
    let any = AnyPolicy::new(left, right);
    assert_ne!(all.root(), any.root());
});
