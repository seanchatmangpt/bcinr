;; Closed-loop controller domain for Claude Code, release v26.7.28.
;;
;; Runs on the EXACT classical rail (ExactClassicalProblem, ground_v2.rs), which
;; is BFS with a visited set. The temporal rail is deliberately not used: it has
;; no closed list, so it restarts every applicable action on every tick -- a
;; measured single-agent episode there produced a 56-step plan that committed 4
;; times and opened 3 pull requests.
;;
;; ---------------------------------------------------------------------------
;; Two design decisions, both forced by measurement rather than taste.
;; ---------------------------------------------------------------------------
;;
;; 1. The governance rules (phase-ready, all-tests-pass,
;;    all-release-phases-complete) are EXPANDED INLINE rather than declared as
;;    (:derived ...). Two independent reasons, both verified in source:
;;
;;      a. ground_v2.rs:322-324 refuses any domain containing :derived outright.
;;      b. The other rail's compute_derived_closure (ground/mod.rs:1963-1985)
;;         only ever inserts derived atoms and never retracts them, and
;;         successors clone the already-closed state forward (:405). A derived
;;         predicate therefore latches: once all-tests-pass held it would keep
;;         holding after admit-verification-failure deleted every test-passed
;;         fact. That is precisely the "permission withdrawn" case governance
;;         exists to enforce.
;;
;;    All three rules are non-recursive, so inlining is semantics-preserving.
;;
;; 2. There is NO workflow object type. An earlier revision modelled a pool of
;;    interchangeable workflow slots that each launch action claimed. Measured
;;    result: one phase solved in 8 steps, but two phases exhausted 1,000,000
;;    search states in 517 seconds. The slots are symmetric -- every launch
;;    branches over every unused slot -- and BFS has no symmetry reduction.
;;
;;    The pool was also redundant. `phase-implementing ?p` already means "an
;;    implementation workflow is in flight for ?p", and `test-running ?p ?s`
;;    already means "a verification workflow is in flight for ?p/?s". Workflow
;;    identity is an execution detail: the controller assigns the real workflow
;;    id when it dispatches, and records it in the OCEL evidence log alongside
;;    the command output. It is not a planning decision, so it does not belong
;;    in planning state.
;;
;; ---------------------------------------------------------------------------
;;
;; This is the OPTIMISTIC model, used for planning only. The admit-* actions
;; here do not require observed-* evidence, so that a complete plan to
;; release-complete exists and BFS is genuinely goal-directed. The controller
;; executes only the first action of the returned plan. Whether an admission is
;; actually permitted is decided against real evidence by the adapter, which
;; emits an admit-* fact only when a command's own output supports it.

(define (domain claude-v26728-controller)

  (:requirements
    :strips
    :typing
    :negative-preconditions
    :disjunctive-preconditions
    :equality
    :quantified-preconditions
    :conditional-effects)

  (:types
    agent
    phase
    test-suite
    release)

  (:predicates
    (available ?a - agent)

    (part-of-release ?p - phase ?r - release)
    (depends-on ?p - phase ?dependency - phase)

    ;; Phase lifecycle. `phase-implementing` doubles as "an implementation or
    ;; repair workflow is in flight".
    (phase-pending ?p - phase)
    (phase-implementing ?p - phase)
    (phase-awaiting-tests ?p - phase)
    (phase-needs-repair ?p - phase)
    (implementation-landed ?p - phase)
    (phase-receipt-sealed ?p - phase)
    (phase-complete ?p - phase)

    ;; `test-running` doubles as "a verification workflow is in flight".
    (required-test ?p - phase ?s - test-suite)
    (test-running ?p - phase ?s - test-suite)
    (test-passed ?p - phase ?s - test-suite)

    (release-pending ?r - release)
    (release-running ?r - release)
    (release-complete ?r - release)

    ;; Exogenous observations. No action in this domain manufactures these --
    ;; the evidence adapter adds them only after reading a command's own output.
    ;;
    ;; They are also what keeps planning tractable. An earlier revision left the
    ;; failure admissions unconditional, so they were applicable alongside their
    ;; success counterparts at every step and BFS explored every
    ;; fail -> repair -> implement -> fail cycle; two phases exhausted 400,000
    ;; states. That was a modelling error, not just a performance one: an
    ;; unconditional failure action means the planner may *choose* to fail.
    ;; Failure is exogenous. Gating on an observation makes these inapplicable
    ;; during planning, and applicable exactly when reality has produced one.
    (observed-workflow-failure ?p - phase)
    (observed-test-fail ?p - phase ?s - test-suite)
    (observed-release-failure ?r - release))

  ;; A phase may start only when every declared dependency is complete.
  ;; (Inlined phase-ready.)
  (:action launch-implementation-workflow
    :parameters (?claude - agent ?p - phase)
    :precondition
      (and
        (available ?claude)
        (phase-pending ?p)
        (forall (?dependency - phase)
          (imply (depends-on ?p ?dependency) (phase-complete ?dependency))))
    :effect
      (and
        (not (phase-pending ?p))
        (phase-implementing ?p)))

  (:action launch-repair-workflow
    :parameters (?claude - agent ?p - phase)
    :precondition
      (and
        (available ?claude)
        (phase-needs-repair ?p))
    :effect
      (and
        (not (phase-needs-repair ?p))
        (phase-implementing ?p)))

  ;; A new implementation invalidates every earlier verification result for the
  ;; phase, and any receipt sealed over them.
  (:action admit-implementation-success
    :parameters (?p - phase)
    :precondition (phase-implementing ?p)
    :effect
      (and
        (not (phase-implementing ?p))
        (implementation-landed ?p)
        (phase-awaiting-tests ?p)
        (forall (?s - test-suite)
          (when (required-test ?p ?s)
            (and (not (test-passed ?p ?s)) (not (test-running ?p ?s)))))
        (not (phase-receipt-sealed ?p))))

  (:action admit-implementation-failure
    :parameters (?p - phase)
    :precondition
      (and
        (phase-implementing ?p)
        (observed-workflow-failure ?p))
    :effect
      (and
        (not (phase-implementing ?p))
        (phase-needs-repair ?p)
        (not (observed-workflow-failure ?p))))

  (:action launch-verification-workflow
    :parameters (?claude - agent ?p - phase ?s - test-suite)
    :precondition
      (and
        (available ?claude)
        (phase-awaiting-tests ?p)
        (implementation-landed ?p)
        (required-test ?p ?s)
        (not (test-running ?p ?s))
        (not (test-passed ?p ?s)))
    :effect (test-running ?p ?s))

  (:action admit-verification-pass
    :parameters (?p - phase ?s - test-suite)
    :precondition
      (and
        (phase-awaiting-tests ?p)
        (implementation-landed ?p)
        (test-running ?p ?s))
    :effect
      (and
        (not (test-running ?p ?s))
        (test-passed ?p ?s)))

  ;; One verifier failure invalidates the whole verification round and any
  ;; receipt sealed over it. Claude must repair and reverify from scratch.
  (:action admit-verification-failure
    :parameters (?p - phase ?s - test-suite)
    :precondition
      (and
        (phase-awaiting-tests ?p)
        (implementation-landed ?p)
        (test-running ?p ?s)
        (observed-test-fail ?p ?s))
    :effect
      (and
        (not (phase-awaiting-tests ?p))
        (not (implementation-landed ?p))
        (not (phase-receipt-sealed ?p))
        (phase-needs-repair ?p)
        (not (observed-test-fail ?p ?s))
        (forall (?suite - test-suite)
          (when (required-test ?p ?suite)
            (and (not (test-passed ?p ?suite)) (not (test-running ?p ?suite)))))))

  ;; A receipt may be sealed only when every required suite has passed.
  ;; (Inlined all-tests-pass.)
  (:action admit-phase-receipt
    :parameters (?p - phase)
    :precondition
      (and
        (phase-awaiting-tests ?p)
        (implementation-landed ?p)
        (forall (?s - test-suite)
          (imply (required-test ?p ?s) (test-passed ?p ?s)))
        (not (phase-receipt-sealed ?p)))
    :effect (phase-receipt-sealed ?p))

  (:action mark-phase-complete
    :parameters (?p - phase)
    :precondition
      (and
        (phase-awaiting-tests ?p)
        (implementation-landed ?p)
        (phase-receipt-sealed ?p)
        (forall (?s - test-suite)
          (imply (required-test ?p ?s) (test-passed ?p ?s)))
        (not (phase-complete ?p)))
    :effect
      (and
        (not (phase-awaiting-tests ?p))
        (not (implementation-landed ?p))
        (phase-complete ?p)))

  ;; The release may publish only after every constituent phase is complete.
  ;; (Inlined all-release-phases-complete.)
  (:action launch-release-workflow
    :parameters (?claude - agent ?r - release)
    :precondition
      (and
        (available ?claude)
        (release-pending ?r)
        (forall (?p - phase)
          (imply (part-of-release ?p ?r) (phase-complete ?p))))
    :effect
      (and
        (not (release-pending ?r))
        (release-running ?r)))

  (:action admit-release-success
    :parameters (?r - release)
    :precondition (release-running ?r)
    :effect
      (and
        (not (release-running ?r))
        (release-complete ?r)))

  (:action admit-release-failure
    :parameters (?r - release)
    :precondition
      (and
        (release-running ?r)
        (observed-release-failure ?r))
    :effect
      (and
        (not (release-running ?r))
        (release-pending ?r)
        (not (observed-release-failure ?r)))))
