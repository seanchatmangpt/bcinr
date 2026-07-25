import re

with open("crates/bcinr-logic/src/autonomic/autonomic_substrate.rs", "r") as f:
    content = f.read()

content = content.replace(
    "pub struct AutonomicSubstrate<K, V, const N: usize>",
    "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub struct AutonomicSubstrate<K, V, const N: usize>"
)

with open("crates/bcinr-logic/src/autonomic/autonomic_substrate.rs", "w") as f:
    f.write(content)
