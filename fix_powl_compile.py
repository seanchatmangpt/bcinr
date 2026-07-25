import re

with open("crates/bcinr-powl/src/full_mapek_loop.rs", "r") as f:
    content = f.read()

# Fix terminal_state rename that was a mistake
content = content.replace(
    "_terminal_state: &mut PersistentControlState",
    "terminal_state: &mut PersistentControlState"
)

# Fix unused_mut in terminal_state in test
content = content.replace(
    "let terminal_state = PersistentControlState::default();",
    "let mut terminal_state = PersistentControlState::default();"
)

# Fix mut in term1, term2, term3, term4
content = content.replace("let term1 = terminal", "let mut term1 = terminal")
content = content.replace("let term2 = terminal", "let mut term2 = terminal")
content = content.replace("let term3 = terminal", "let mut term3 = terminal")
content = content.replace("let term4 = terminal", "let mut term4 = terminal")

with open("crates/bcinr-powl/src/full_mapek_loop.rs", "w") as f:
    f.write(content)
