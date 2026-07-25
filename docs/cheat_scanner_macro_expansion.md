# `bcinr-cheat-scanner`: Macro Expansion and Evasion Detection

## Why Checking Raw Source AST is Insufficient

According to Rule 17 of the BCINR Constitution, the `bcinr-cheat-scanner` must "inspect macro definitions and expanded output." Relying solely on the raw source Abstract Syntax Tree (AST) before macro expansion is a critical vulnerability. 

In Rust, macros (both declarative `macro_rules!` and procedural macros) operate as syntactic abstractions. If a scanner only inspects the pre-expansion AST, a macro invocation appears simply as a `MacCall` node. The AST for the calling function contains no `ExprKind::If`, `ExprKind::Match`, or `ExprKind::Loop` nodes. Thus, a naive scanner checking for `CC=1` (Cyclomatic Complexity = 1) at the source level will incorrectly certify the function as branchless, completely missing the control flow hidden behind the macro boundary. 

This violates the mandate that branchlessness must apply to the transitively generated source and the entire authoritative call graph (Rule 7, Rule 8).

## Example: Hiding Branches and Loops via Macro Indirection

An adversary (or an agent prioritizing implementation convenience over constitutional laws) might attempt to bypass the `CC=1` restriction by defining a macro that encapsulates prohibited control flow (CHEAT-006: "using macro indirection to hide a pattern"):

### Hiding an `if` statement:
```rust
macro_rules! fake_branchless_select {
    ($cond:expr, $true_val:expr, $false_val:expr) => {
        // CHEAT-006: Prohibited operator hidden in macro expansion
        if $cond {
            $true_val
        } else {
            $false_val
        }
    };
}

pub fn update_state(admitted: bool, candidate: u64, current: u64) -> u64 {
    // The raw AST here sees NO branch. It only sees a `MacCall`.
    fake_branchless_select!(admitted, candidate, current)
}
```

### Hiding a loop backedge:
```rust
macro_rules! fake_unrolled_search {
    ($slice:expr) => {{
        let mut found = 0;
        // CHEAT-006: Variable-bound iteration hidden in macro
        while found < $slice.len() {
            if $slice[found] == 0 {
                break; // Prohibited control flow
            }
            found += 1;
        }
        found
    }};
}

pub fn find_zero(data: &[u64]) -> usize {
    // Looks like straight-line code to a pre-expansion scanner
    fake_unrolled_search!(data)
}
```

## How Expansion Catches the Evasion

By hooking into the compiler's expansion phase (e.g., via `rustc` internals or generating expanded source code), the `bcinr-cheat-scanner` analyzes the *fully expanded* AST. 

When the macro is expanded, `fake_branchless_select!(admitted, candidate, current)` is replaced by its literal expansion:

```rust
pub fn update_state(admitted: bool, candidate: u64, current: u64) -> u64 {
    if admitted {
        candidate
    } else {
        current
    }
}
```

The expanded AST now correctly surfaces the `if` expression (e.g., `ExprKind::If`). The scanner will immediately detect this node, properly flag it as a violation of Rule 8 (Absolute `CC=1` law), and identify the attempt as CHEAT-006 (Scanner evasion). This forces the developer to rewrite the logic using legitimate bitwise polynomials, masks, and fixed-width state transitions (e.g., `(mask & candidate) | (!mask & current)`) as mandated by Rule 9.
