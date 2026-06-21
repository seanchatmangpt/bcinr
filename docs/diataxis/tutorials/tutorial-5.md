# Tutorial 5: A Tiny Branchless State Machine

Parsers, protocol validators, and tokenizers are usually a thicket of `match` and
`if` statements — exactly the data-dependent branches that wreck pipeline
predictability. The `dfa` module replaces all of that with one array lookup per
byte. In this tutorial you build a real, working DFA that recognizes a simple
pattern, with no branch in the hot loop.

## What you'll build

A deterministic finite automaton that accepts byte strings of the form
`a+b` (one or more `a`s followed by a single `b`), driven entirely by
`dfa::dfa_advance`, `dfa::dfa_run`, and `dfa::dfa_is_accepting`.

**Prerequisites:** [Tutorial 1](./tutorial-1.md). Knowing what a state machine is
helps, but we build the table from scratch.

## Step 1: Understand the transition kernel

From `crates/bcinr-logic/src/dfa.rs`:

```rust
pub fn dfa_advance(state: usize, input: u8, table: &[usize], alphabet_size: usize) -> usize {
    let index = state.wrapping_mul(alphabet_size).wrapping_add(input as usize);
    let mask = (index < table.len()) as usize;
    let safe_idx = index & (0usize.wrapping_sub(mask));
    table[safe_idx] & (0usize.wrapping_sub(mask))
}
```

The next state is `table[state * alphabet_size + input]`. The `mask` makes an
out-of-bounds index fold to `0` instead of panicking — branchless bounds safety,
just like the bitset kernels in [Tutorial 4](./tutorial-4.md).

## Step 2: Design the states

We only care about three byte classes, so we use a full 256-symbol alphabet and a
dead (trap) state for anything unexpected.

| State | Meaning                              |
|-------|--------------------------------------|
| 0     | start (no input yet)                 |
| 1     | seen one or more `a`                 |
| 2     | seen `a+` then `b` — **accepting**   |
| 3     | dead / reject (sticky)               |

Transitions:

- From **0**: `a` -> 1, anything else -> 3
- From **1**: `a` -> 1, `b` -> 2, anything else -> 3
- From **2**: any further byte -> 3 (the `b` must be last)
- From **3**: stays 3 forever

## Step 3: Build the transition table

The table is row-major: row `s` occupies indices `s * 256 .. s * 256 + 256`.

```rust
const ALPHABET: usize = 256;
const N_STATES: usize = 4;
const DEAD: usize = 3;

fn build_table() -> [usize; N_STATES * ALPHABET] {
    // every transition defaults to the dead state...
    let mut table = [DEAD; N_STATES * ALPHABET];

    // ...then we carve out the accepting paths.
    table[0 * ALPHABET + b'a' as usize] = 1; // start --a--> 1
    table[1 * ALPHABET + b'a' as usize] = 1; // 1     --a--> 1 (one or more)
    table[1 * ALPHABET + b'b' as usize] = 2; // 1     --b--> 2 (accept)
    // state 2 and 3 fall through to DEAD for every byte.

    table
}
```

## Step 4: Run the DFA over input

```rust
use bcinr_logic::dfa::{dfa_is_accepting, dfa_run};

fn accepts(input: &[u8]) -> bool {
    let table = build_table();
    let final_state = dfa_run(&table, ALPHABET, /* initial_state */ 0, input);
    dfa_is_accepting(final_state, /* accept_states */ &[2])
}

fn main() {
    for s in ["ab", "aaab", "b", "aa", "abc", ""] {
        println!("{:>5} -> {}", s, accepts(s.as_bytes()));
    }
}
```

(`build_table`, `ALPHABET`, `N_STATES`, and `DEAD` come from Step 3.)

## Step 5: Run it

```bash
cargo run
```

Expected output:

```
   ab -> true
 aaab -> true
    b -> false
   aa -> false
  abc -> false
      -> false
```

`"ab"` and `"aaab"` match `a+b`; the rest fall into the dead state and are
rejected. The empty string ends in the start state, which is not accepting.

## Step 6: Step one byte at a time

`dfa_run` is just a loop over `dfa_advance`. You can drive the machine manually to
inspect intermediate states — handy for streaming input:

```rust
use bcinr_logic::dfa::dfa_advance;

fn main() {
    let table = build_table();
    let mut state = 0usize;
    for &byte in b"aab" {
        state = dfa_advance(state, byte, &table, ALPHABET);
        println!("after {:?}: state {}", byte as char, state);
    }
}
```

```bash
cargo run
```

Expected output:

```
after 'a': state 1
after 'a': state 1
after 'b': state 2
```

Every byte costs the same: one multiply, one add, one masked load. The control
flow is identical whether the input matches or not — which is exactly what makes
DFAs side-channel friendly.

## Step 7: Lock it in

```rust
use bcinr_logic::dfa::{dfa_is_accepting, dfa_run};

#[test]
fn dfa_recognizes_a_plus_b() {
    let table = build_table();
    let run = |s: &str| {
        let end = dfa_run(&table, ALPHABET, 0, s.as_bytes());
        dfa_is_accepting(end, &[2])
    };
    assert!(run("ab"));
    assert!(run("aaaab"));
    assert!(!run("abb")); // trailing byte after the b -> dead
    assert!(!run("ba"));
}
```

```bash
cargo test dfa_recognizes_a_plus_b
```

```
test dfa_recognizes_a_plus_b ... ok
```

## What you learned

- A DFA reduces parsing to one masked array lookup per byte via `dfa_advance`.
- The transition table is row-major: `table[state * alphabet_size + byte]`.
- A sticky dead state plus `dfa_is_accepting` gives you full
  pattern-recognition with no branch in the loop.

## Next steps

- [Tutorial 7: Branchless scans over byte slices](./tutorial-7.md) — pre-classify
  bytes in bulk before feeding them to a DFA.
- [Tutorial 9: Property-testing a branchless kernel](./tutorial-9.md) — verify a
  DFA against a branchful reference for all inputs.
