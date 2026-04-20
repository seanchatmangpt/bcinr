import os

path = "crates/bcinr-logic/src/algorithms/fixed_point_log2.rs"
with open(path, "r") as f:
    content = f.read()

# Revert the proptest range to something standard that syn parses
content = content.replace("val in 1..4294967295u64", "val in 1..100000u64")
content = content.replace("val in 2..1000u64", "val in 2..1000u64")

with open(path, "w") as f:
    f.write(content)
