# Reference: Algorithm Families

## 1. Bitset Algebra
Tools for managing massive sparse sets.
- `rank`: Cardinality prefix-sum.
- `select`: Inverse mapping.

## 2. Saturation Arithmetic
Operations that clamp to boundaries.
- `add_sat`, `sub_sat`: Essential for physical thrust and thermal limits.

## 3. DFA (Deterministic Finite Automata)
Table-driven state machines for line-rate parsing.
- `dfa_advance`: Atomic transition primitive.
