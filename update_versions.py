import os
import glob

def update_version(path):
    with open(path, "r") as f:
        content = f.read()
    new_content = content.replace('"26.4.22"', '"26.6.12"').replace('"26.4.19"', '"26.6.12"')
    if content != new_content:
        with open(path, "w") as f:
            f.write(new_content)
        print(f"Updated {path}")

for root, dirs, files in os.walk("."):
    for file in files:
        if file == "Cargo.toml":
            update_version(os.path.join(root, file))

update_version("Cargo.lock")
update_version("GEMINI.md")

