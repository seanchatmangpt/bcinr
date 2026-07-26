# PDDL Subsystem Contract

**Version:** 26.7.17  
**Scope:** BranchlessCInRust PDDL 3.1 Planner  
**Responsibility:** Domain parsing, problem grounding, classical backward-chaining search

## Preconditions

### Input Domain (Pre-Parse)
```
∀ domain ∈ InputDomainText:
  - Well-formed S-expression syntax (Lisp parentheses)
  - Named predicates with arities: (:predicates (p ?x ?y) (q ?z))
  - Actions or durative-actions with:
    • Parameters: typed or untyped variables
    • Precondition: PddlCondition (and, or, not, forall, exists, etc.)
    • Effect: list of PddlEffect (add/delete propositions, numeric updates)
  - Types (optional, :typing requirement): partitions object universe
  - Requirements subset of: [:strips, :typing, :adl, :durative-actions, :numeric-fluents, :constraints]
```

### Input Problem (Pre-Ground)
```
∀ problem ∈ InputProblemText:
  - Refers to existing domain by name
  - Objects: concrete instances with optional type assignments
  - Init state: list of ground atoms (p a b) or numeric assignments (= (f a) 5.0)
  - Goal: ground condition tree (and, or, not, forall, etc.)
  - Optional: metric, preferences, trajectory constraints
```

### Planning Bounds (Pre-Search)
```
∀ bounds ∈ PlanningBounds:
  - max_ground_actions ∈ [1, 2^16): upper bound on grounded action schema instances
  - max_plan_depth ∈ [1, 2^16): upper bound on plan length (depth in backward search)
  - max_search_steps ∈ [1, 2^32): upper bound on goal-reachability BFS iterations
  - max_partition_boxes ∈ [1, 16]: partitioning strategy limit for massive domains
```

## Postconditions

### Successful Parse → Domain31
```
domain31 ∈ Pddl31Domain ⟹
  ✓ domain31.name = name given in (define (domain ...))
  ✓ domain31.requirements = normalized, deduplicated requirement set
  ✓ domain31.predicates: ∀ pred ∈ predicates, arity(pred) ≥ 0
  ✓ domain31.actions: ∀ action ∈ actions, action.params well-typed
  ✓ domain31.durative_actions: ∀ da ∈ durative_actions, da.duration ∈ TimeSpec
  ✓ No undefined predicate or function references within action bodies
  ✓ Parse deterministic: P(domain_text) = same domain31 object on every invocation
```

### Successful Parse → Problem31
```
problem31 ∈ Pddl31Problem ⟹
  ✓ problem31.domain_name matches the domain's (define (domain X))
  ✓ problem31.objects: ∀ obj, obj.type ⟹ type declared in domain
  ✓ problem31.init_atoms: ground facts; ∀ atom, arity(atom.pred) = declared arity
  ✓ problem31.init_fn_values: ∀ (function, value), function well-typed, value ∈ ℝ
  ✓ problem31.goal: well-formed PddlCondition tree
  ✓ Parse deterministic: P(problem_text) = same problem31 object on every invocation
```

### Successful Ground
```
ground_problem ∈ GroundProblem ⟹
  ✓ ground_problem.initial_state: canonical bit-packed set of ground atoms
  ✓ ground_problem.goal: DNF or CNF normal form, ground ⟹ no free variables
  ✓ ground_problem.actions: ∀ grounded_action ∈ actions:
      • Precondition: ∀ atom in precondition, atom ∈ GroundAtom
      • Effect: ∀ add/del, ground propositions only
      • |ground_problem.actions| ≤ bounds.max_ground_actions
  ✓ Grounding deterministic: same domain/problem ⟹ isomorphic GroundProblem
```

### Successful Plan Search → Pddl8Tape
```
tape ∈ Pddl8Tape ⟹
  ✓ tape.goal_reached: goal condition satisfied in final state
  ✓ tape.ops: ordered sequence of action labels [action_1, ..., action_k]
  ✓ |tape.ops| ≤ bounds.max_plan_depth
  ✓ ∀ i ∈ [0, |ops|):
      • ops[i].precondition ⊆ state_after(ops[i-1])  [i > 0, else state_after(init)]
      • state_after(ops[i]) = state_after(ops[i-1]) ∪ effects(ops[i]) \ deleted_facts(ops[i])
  ✓ search_iterations ≤ bounds.max_search_steps
  ✓ Plan deterministic: same domain/problem/bounds ⟹ identical tape (same action sequence)
```

## Invariants

### Maintained Throughout Execution

1. **No Type Violations**
   ```
   ∀ action invocation, ∀ parameter p:
     type(p) ⊆ declared_type_in_domain(p)
   ```

2. **Precondition Closure**
   ```
   ∀ step_i in plan:
     precondition(step_i) ⊆ state_before(step_i)
   ```

3. **Effect Determinism**
   ```
   ∀ step_i invoked twice with same bindings:
     effects(step_i) = effects(step_i) [idempotent]
   ```

4. **Goal Reachability**
   ```
   if tape.goal_reached = true ⟹
     ∀ goal_atom ∈ goal:
       goal_atom ∈ state_after(final_step)
   ```

5. **No Undefined Predicates**
   ```
   ∀ atom ∈ state ∪ precondition ∪ effect:
     atom.pred ∈ domain.predicates
   ```

6. **Deterministic Execution Path**
   ```
   search_policy(domain, problem, bounds) is deterministic:
     Same inputs ⟹ identical action sequence (modulo UNSAT)
   ```

## Refusal Conditions

Typed refusal enumeration: `enum Pddl8Error { ... }`

### Parse Errors (Pddl8Error::ParseError)
```
| Missing domain name
| S-expression malformed (unclosed paren, invalid syntax)
| Invalid requirement keyword not in [:strips, :typing, :adl, ...]
| Duplicate predicate name
| Action missing :precondition or :effect
| Durative action missing :duration
| Invalid type annotation (e.g., ?x - undefined_type)
| Circular type inheritance
```

### Grounding Errors (ExactClassicalError::*) 
```
| Undefined object in problem (:objects a, b; used c)
| Action parameter type mismatch (action expects truck, given package)
| Negation as failure over undefined predicates
| Precondition unsatisfiable (e.g., (and (p) (not (p))) in precondition)
| Goal ground size exceeds bounds.max_ground_actions
```

### Planning Errors (Pddl8Error::NoPlan / ExactClassicalError::NoPlan)
```
| Goal unreachable from initial state under action set
| Plan depth exceeds bounds.max_plan_depth
| Search iterations exceed bounds.max_search_steps
| Trajectory constraint violated (always, sometime, etc.)
| Numeric constraint violated (fluid quantity underflows, e.g., fuel < 0)
```

### Admission Errors (AdmissionError::*)
```
| Domain uses unsupported feature (e.g., :object-functions without numeric-fluents)
| Problem domain name does not match parsed domain name
| Cyclic action dependencies (should not occur in STRIPS, but checked in ADL)
| Resource constraint violation at ground time (too many objects for partition boxes)
```

## Semantics

### Classical STRIPS Planning (Requirements: :strips)
- Deterministic state transitions: `s' = (s \ delete_list) ∪ add_list`
- Backward chaining BFS from goal state to initial state
- Plan linear; no concurrency
- **Plan is sound:** every precondition provable at execution time
- **Plan is complete (within bounds):** if reachable, found within max_search_steps

### Typing Constraint (Requirement: :typing)
- Each object assigned a type
- Action parameters typed: `(?x - location ?y - package)`
- Type inheritance via `:types vehicle (car truck)`
- **Invariant:** ∀ atom, all arguments match declared types

### Numeric Fluents (Requirement: :numeric-fluents)
- Functions: `(fuel truck1) → ℝ`
- Effects: `(increase (fuel ?t) 5.5)`, `(decrease (fuel ?t) 2)`, `(assign (fuel ?t) 10)`
- Preconditions: `(>= (fuel ?t) 5)`
- **Invariant:** numeric expressions preserve type; no division by zero

### ADL / Quantification (Requirement: :adl)
- `(forall (?x - T) cond)`: ∀ bindings of ?x with type T, condition must hold
- `(exists (?x - T) cond)`: ∃ binding of ?x with type T such that condition holds
- Negation as failure: `(not (p ?x))`
- **Invariant:** quantified bodies well-scoped; no free variables

### Durative Actions (Requirement: :durative-actions)
- Duration: `(= ?duration 5)` or `(<= ?duration 10)`
- Conditions: `(at start (p))`, `(over all (q))`, `(at end (r))`
- Effects at timed points: `(at end (effect))`
- **Invariant:** durative plan respects temporal ordering; concurrent actions respect resource constraints

### Trajectory Constraints (Requirement: :constraints)
- `(always (not (p)))`: p must never hold
- `(sometime (q))`: q must hold at some point
- `(within 5 (r))`: r must hold within 5 time units
- Checked over entire plan trace
- **Refusal:** Pddl8Error::ConstraintViolation if any violated

## Examples

### Minimal Solvable Problem
```pddl
(define (domain simple)
  (:requirements :strips)
  (:predicates (p) (q))
  (:action make-q
    :precondition (p)
    :effect (q)))

(define (problem sp)
  (:domain simple)
  (:init (p))
  (:goal (q)))
```
**Expected:** `tape.ops = ["make-q"], tape.goal_reached = true`

### Type Mismatch Refusal
```pddl
(define (domain typed)
  (:requirements :typing)
  (:types location package)
  (:predicates (at ?x - package ?l - location))
  (:action move :parameters (?p - package ?l - location)
    :precondition (at ?p ?l)
    :effect (at ?p ?l)))

(define (problem bad-types)
  (:domain typed)
  (:objects loc1 - location)
  (:init (at loc1 loc1))  ;; ERROR: first arg must be package
  (:goal (at loc1 loc1)))
```
**Expected:** `Pddl8Error::ParseError` or `ExactClassicalError::TypeMismatch`

### Unreachable Goal
```pddl
(define (domain unreachable)
  (:requirements :strips)
  (:predicates (locked) (unlocked))
  (:action unlock
    :precondition (locked)
    :effect (unlocked)))

(define (problem locked)
  (:domain unreachable)
  (:init (locked))
  (:goal (unlocked))
  (:constraints (always (locked))))  ;; CONFLICT: always locked but goal requires unlocked
```
**Expected:** `Pddl8Error::ConstraintViolation` or `ExactClassicalError::NoPlan`

## Standing

- **Scope:** Semantic admission of planning domains/problems; classical deterministic planning
- **Cyclomatic Complexity:** CC ≤ 2 per function (branching for success/error only)
- **Allocation:** Stack-based during parse; heap allocations tracked in `GroundProblem::build`
- **Determinism:** STRIPS + typing planning is fully deterministic; ADL/quantification adds minor nondeterminism only in tie-breaking (sorted by action name)
- **Proof:** Hoare-logic contracts in `HOARE_TRIPLES.md`; oracles in `crates/bcinr-pddl/tests/`
