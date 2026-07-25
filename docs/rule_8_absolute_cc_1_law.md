Here is the detailed list of prohibited Rust constructs that produce control-flow branches under Rule 8 ("Absolute `CC=1` law") in `AGENTS.md`:

```text
if
if let
else
match
while
loop
break
continue
early return
?
unwrap
unwrap_or
unwrap_or_else
expect
checked arithmetic with branch-bearing handling
Option-based control flow
Result-based control flow
iterator short-circuiting
variable-bound iteration
bounds-check panic paths
```

Additionally, the rule stipulates that the following also count as prohibited branches:
* Macro-generated branches
* Branches hidden in trait implementations
* Branches hidden in dependencies if reachable from the authoritative call graph

The scanner must inspect the parsed syntax tree rather than only source lines, and private wrappers do not reduce the complexity standing.
