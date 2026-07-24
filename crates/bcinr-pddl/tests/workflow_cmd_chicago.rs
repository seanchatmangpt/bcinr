//! Chicago TDD acceptance tests for the CMD workflow compiler surface.
//!
//! These tests exercise state and externally visible behavior with real
//! collaborators from planning through POWL verification. The only test double
//! is the recording broker at the explicit actuation boundary.

#![cfg(feature = "mfw-planner")]

use std::borrow::Cow;

use bcinr_pddl::prelude::*;
use serde::Serialize;

const DOMAIN: &str = r#"
(define (domain fulfillment)
  (:requirements :strips)
  (:predicates (paid) (reserved) (notified))
  (:action reserve-inventory
    :parameters ()
    :precondition (paid)
    :effect (reserved))
  (:action notify-customer
    :parameters ()
    :precondition (paid)
    :effect (notified)))
"#;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct OrderState {
    order_id: u64,
    paid: bool,
}

impl WorkflowProblem for OrderState {
    fn to_pddl_problem(&self) -> Cow<'_, str> {
        let init = if self.paid { "(paid)" } else { "" };
        Cow::Owned(format!(
            "(define (problem order-{id})\n  (:domain fulfillment)\n  (:init {init})\n  (:goal (and (reserved) (notified))))",
            id = self.order_id,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
enum FulfillmentCommand {
    ReserveInventory,
    NotifyCustomer,
}

#[derive(Debug, Clone, Copy)]
struct FulfillmentBinding;

impl ActionBinding for FulfillmentBinding {
    type Command = FulfillmentCommand;
    type Error = String;

    fn binding_name(&self) -> &str {
        "fulfillment-command-v1"
    }

    fn supported_actions(&self) -> Vec<&str> {
        vec!["reserve-inventory", "notify-customer"]
    }

    fn bind(&self, action: &ActionInvocation) -> Result<Self::Command, Self::Error> {
        match (action.name.as_str(), action.arguments.as_slice()) {
            ("reserve-inventory", []) => Ok(FulfillmentCommand::ReserveInventory),
            ("notify-customer", []) => Ok(FulfillmentCommand::NotifyCustomer),
            _ => Err(format!("unbound workflow action: {}", action.label)),
        }
    }
}

#[derive(Debug, Clone)]
struct TenantPolicy {
    allowed_tenant: &'static str,
}

impl PolicyIdentity for TenantPolicy {
    fn root(&self) -> PolicySetRoot {
        PolicySetRoot::hash_parts(&[b"tenant-policy-v1", self.allowed_tenant.as_bytes()])
    }
}

impl Policy<TypedWorkflowPlan<FulfillmentCommand>, ApplicationContext> for TenantPolicy {
    type Evidence = &'static str;
    type Refusal = &'static str;

    fn evaluate(
        &self,
        _input: &TypedWorkflowPlan<FulfillmentCommand>,
        context: &ApplicationContext,
    ) -> PolicyDecision<Self::Evidence, Self::Refusal> {
        if context.tenant == self.allowed_tenant {
            PolicyDecision::Admit("tenant-admitted")
        } else {
            PolicyDecision::Refuse("tenant-refused")
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct CommandLimitPolicy {
    max_commands: usize,
}

impl PolicyIdentity for CommandLimitPolicy {
    fn root(&self) -> PolicySetRoot {
        PolicySetRoot::hash_parts(&[b"command-limit-policy-v1", &self.max_commands.to_le_bytes()])
    }
}

impl Policy<TypedWorkflowPlan<FulfillmentCommand>, ApplicationContext> for CommandLimitPolicy {
    type Evidence = usize;
    type Refusal = usize;

    fn evaluate(
        &self,
        input: &TypedWorkflowPlan<FulfillmentCommand>,
        _context: &ApplicationContext,
    ) -> PolicyDecision<Self::Evidence, Self::Refusal> {
        let count = input
            .batches()
            .iter()
            .map(|batch| batch.actions().len())
            .sum::<usize>();
        if count <= self.max_commands {
            PolicyDecision::Admit(count)
        } else {
            PolicyDecision::Refuse(count)
        }
    }
}

#[derive(Debug)]
struct ApplicationContext {
    tenant: &'static str,
}

fn arrange_compiled_workflow() -> (
    EmbeddedWorkflow,
    OrderState,
    ObservationSnapshot<OrderState>,
    GoalEnvelope<GoalExpr<String, i64>>,
    BindingRegistry<FulfillmentBinding>,
) {
    let workflow = EmbeddedWorkflow::new(DOMAIN).expect("domain should install");
    let state = OrderState {
        order_id: 42,
        paid: true,
    };
    let observation = ObservationSnapshot::manufacture(
        LogicalTime(7),
        SourceVersion("orders-table:17".to_string()),
        state.clone(),
    )
    .expect("observation should canonicalize");
    let goal = GoalEnvelope::manufacture(
        GoalExpr::<String, i64>::All(vec![
            GoalExpr::Atom("reserved".to_string()),
            GoalExpr::Atom("notified".to_string()),
        ]),
        GoalPriority(100),
        None,
        GoalPolicy::Hard,
    )
    .expect("goal should canonicalize");
    let binding = BindingRegistry::new(FulfillmentBinding).expect("binding schema should validate");
    (workflow, state, observation, goal, binding)
}

chicago_tdd_tools::test!(
    application_compiles_control_flow_and_advances_only_after_observed_effects,
    {
        // Arrange: install the real domain and application collaborators.
        let (mut workflow, state, observation, goal, binding) = arrange_compiled_workflow();
        let coverage = binding.coverage(["reserve-inventory", "notify-customer"]);
        assert!(coverage.is_complete());

        // Act: plan, verify, bind, authorize, propose, admit, and observe effects.
        let verified = workflow
            .plan(&state)
            .expect("planning should earn standing");
        let typed = binding
            .bind_plan(&verified)
            .expect("every planner action should bind");
        let envelope = workflow
            .manufacture_plan_envelope(
                &verified,
                observation.root(),
                goal.root(),
                PlanningBounds::interactive(),
                SearchPolicyRoot::hash(b"first-valid-deterministic-v1"),
            )
            .expect("verified roots should form a plan envelope");

        let policies = AllPolicy::new(
            TenantPolicy {
                allowed_tenant: "acme",
            },
            CommandLimitPolicy { max_commands: 2 },
        );
        let context = ApplicationContext { tenant: "acme" };
        let evidence = match policies.evaluate(&typed, &context) {
            PolicyDecision::Admit(evidence) => evidence,
            PolicyDecision::Refuse(refusal) => panic!("policy refused valid work: {refusal:?}"),
        };
        assert_eq!(evidence, ("tenant-admitted", 2));

        let proposal = DispatchProposal::from_typed_plan(
            &typed,
            &envelope,
            binding.schema_root(),
            Some(policies.root()),
            IdempotencyKey::new("order-42:generation-0").expect("idempotency key"),
        )
        .expect("typed work should manufacture a proposal");
        let mut broker = RecordingBroker::default();
        let admission = broker
            .admit_batch(&proposal)
            .expect("broker should admit the first proposal");
        let mut cursor = WorkflowCursor::from_proposal(&proposal);
        let ready = cursor.next_ready();
        cursor
            .record_admission(&admission)
            .expect("admission should match the cursor proposal");
        for (tick, command_index) in ready {
            cursor
                .record_effect(
                    tick,
                    command_index,
                    EffectRoot::hash_parts(&[
                        b"observed-effect",
                        &tick.to_le_bytes(),
                        &command_index.to_le_bytes(),
                    ]),
                )
                .expect("admitted commands should accept observed effects");
        }

        // Assert: real state changes at every boundary remain visible and distinct.
        assert_eq!(
            typed.standing(),
            CognitiveExecutionStanding::WitnessedConcurrentStrips
        );
        assert_eq!(proposal.commands().len(), 2);
        assert_eq!(broker.admitted_roots(), &[proposal.root()]);
        assert!(cursor.next_tick().is_none());
        assert!(cursor
            .commands()
            .iter()
            .all(|command| matches!(command.progress, CommandProgress::EffectObserved { .. })));
    }
);

chicago_tdd_tools::test!(transport_replay_restores_only_matching_trusted_evidence, {
    // Arrange: manufacture a real verified plan and erase its standing for transport.
    let (mut workflow, state, observation, goal, _binding) = arrange_compiled_workflow();
    let verified = workflow.plan(&state).expect("planning should succeed");
    let trusted = workflow
        .manufacture_plan_envelope(
            &verified,
            observation.root(),
            goal.root(),
            PlanningBounds::interactive(),
            SearchPolicyRoot::hash(b"first-valid-deterministic-v1"),
        )
        .expect("trusted plan envelope");
    let json =
        serde_json::to_string(&trusted.erase_for_transport()).expect("transport should serialize");

    // Act: deserialize as explicitly untrusted data and replay against evidence.
    let transported: UntrustedPlanEnvelope =
        serde_json::from_str(&json).expect("transport shape should deserialize");
    let restored = transported
        .verify_against(&trusted)
        .expect("complete matching evidence should restore standing");

    // Assert: the restored plan is exact, while tampered data is refused.
    assert_eq!(restored, trusted);
    let mut tampered = trusted.erase_for_transport();
    tampered.claimed_plan_root = PlanRoot::hash(b"forged-plan");
    assert!(matches!(
        tampered.verify_against(&trusted),
        Err(TransportTrustError::PlanRootMismatch)
    ));
});

chicago_tdd_tools::test!(
    receipt_chain_connects_decision_dispatch_effect_and_cursor,
    {
        // Arrange: compile and broker a real application plan.
        let (mut workflow, state, observation, goal, binding) = arrange_compiled_workflow();
        let verified = workflow.plan(&state).expect("planning should succeed");
        let typed = binding
            .bind_plan(&verified)
            .expect("binding should succeed");
        let envelope = workflow
            .manufacture_plan_envelope(
                &verified,
                observation.root(),
                goal.root(),
                PlanningBounds::interactive(),
                SearchPolicyRoot::hash(b"first-valid-deterministic-v1"),
            )
            .expect("plan envelope");
        let proposal = DispatchProposal::from_typed_plan(
            &typed,
            &envelope,
            binding.schema_root(),
            None,
            IdempotencyKey::new("order-42:receipts").expect("idempotency key"),
        )
        .expect("dispatch proposal");
        let mut broker = RecordingBroker::default();
        let admission = broker.admit_batch(&proposal).expect("broker admission");
        let mut cursor = WorkflowCursor::from_proposal(&proposal);
        let ready = cursor.next_ready();
        cursor
            .record_admission(&admission)
            .expect("cursor admission");
        let effect_root = EffectRoot::hash(b"inventory-and-notification-observed");
        for (tick, command_index) in ready {
            cursor
                .record_effect(tick, command_index, effect_root)
                .expect("effect should advance command state");
        }

        // Act: append the cross-boundary receipt ancestry.
        let mut receipts = WorkflowReceiptChain::default();
        receipts.append(ReceiptSubject::Observation(observation.root()));
        receipts.append(ReceiptSubject::Goal(goal.root()));
        receipts.append(ReceiptSubject::Plan(envelope.plan_root()));
        receipts.append(ReceiptSubject::Process(envelope.process_root()));
        receipts.append(ReceiptSubject::Execution(envelope.execution_root()));
        receipts.append(ReceiptSubject::Binding(binding.schema_root()));
        receipts.append(ReceiptSubject::Dispatch(proposal.root()));
        receipts.append(ReceiptSubject::Effect(effect_root));
        receipts.append(ReceiptSubject::Cursor(cursor.root()));

        // Assert: every receipt has one deterministic parent and the chain replays.
        receipts.verify().expect("receipt ancestry should replay");
        assert_eq!(receipts.records().len(), 9);
        assert!(receipts.root().is_some());
        assert_eq!(
            receipts.records()[1].parent,
            Some(receipts.records()[0].root)
        );
    }
);

chicago_tdd_tools::test!(
    residual_reconciliation_preserves_only_unchanged_observations,
    {
        // Arrange: identify the original plan and observation.
        let original_observation = ObservationRoot::hash(b"state-v1");
        let original_plan = PlanRoot::hash(b"plan-v1");

        // Act: reconcile unchanged, changed, satisfied, and over-bound cases.
        let keep = reconcile_residual(&ResidualRequest {
            original_plan,
            original_observation,
            current_observation: original_observation,
            next_tick: Some(3),
            goal_already_satisfied: false,
            generation: 0,
            max_generations: 4,
        });
        let replace = reconcile_residual(&ResidualRequest {
            original_plan,
            original_observation,
            current_observation: ObservationRoot::hash(b"state-v2"),
            next_tick: Some(3),
            goal_already_satisfied: false,
            generation: 1,
            max_generations: 4,
        });
        let satisfied = reconcile_residual(&ResidualRequest {
            original_plan,
            original_observation,
            current_observation: ObservationRoot::hash(b"state-v2"),
            next_tick: Some(3),
            goal_already_satisfied: true,
            generation: 1,
            max_generations: 4,
        });
        let refused = reconcile_residual(&ResidualRequest {
            original_plan,
            original_observation,
            current_observation: original_observation,
            next_tick: Some(3),
            goal_already_satisfied: false,
            generation: 4,
            max_generations: 4,
        });

        // Assert: no stale suffix is silently represented as valid.
        assert_eq!(keep, ReplanDecision::KeepSuffix { from_tick: Some(3) });
        assert!(matches!(replace, ReplanDecision::ReplaceRequired { .. }));
        assert_eq!(satisfied, ReplanDecision::GoalAlreadySatisfied);
        assert!(matches!(
            refused,
            ReplanDecision::Refuse {
                reason: ReplanRefusal::GenerationBoundExceeded { limit: 4 }
            }
        ));
    }
);

chicago_tdd_tools::test!(binding_broker_and_cursor_refuse_boundary_confusion, {
    // Arrange: create an incomplete binding catalog and one real dispatch proposal.
    let (mut workflow, state, observation, goal, binding) = arrange_compiled_workflow();
    let coverage = binding.coverage(["reserve-inventory", "notify-customer", "charge-payment"]);
    let verified = workflow.plan(&state).expect("planning should succeed");
    let typed = binding
        .bind_plan(&verified)
        .expect("binding should succeed");
    let envelope = workflow
        .manufacture_plan_envelope(
            &verified,
            observation.root(),
            goal.root(),
            PlanningBounds::interactive(),
            SearchPolicyRoot::hash(b"first-valid-deterministic-v1"),
        )
        .expect("plan envelope");
    let proposal = DispatchProposal::from_typed_plan(
        &typed,
        &envelope,
        binding.schema_root(),
        None,
        IdempotencyKey::new("order-42:duplicate-check").expect("idempotency key"),
    )
    .expect("proposal");
    let mut broker = RecordingBroker::default();
    let _first = broker.admit_batch(&proposal).expect("first admission");
    let duplicate = broker.admit_batch(&proposal);
    let second_proposal = DispatchProposal::from_typed_plan(
        &typed,
        &envelope,
        binding.schema_root(),
        None,
        IdempotencyKey::new("order-42:different-dispatch").expect("idempotency key"),
    )
    .expect("second proposal");
    let wrong_admission = broker
        .admit_batch(&second_proposal)
        .expect("second distinct proposal should be admitted");
    let mut cursor = WorkflowCursor::from_proposal(&proposal);

    // Act: try to use evidence from the wrong dispatch root.
    let cursor_result = cursor.record_admission(&wrong_admission);

    // Assert: missing bindings, duplicate submissions, and root mismatch stay distinct.
    assert_eq!(coverage.missing_bindings, vec!["charge-payment"]);
    assert!(matches!(
        duplicate,
        Err(RecordingBrokerRefusal::DuplicateIdempotency(_))
    ));
    assert_eq!(cursor_result, Err(CursorError::DispatchRootMismatch));
});
