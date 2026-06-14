
import os
import re

def fix_files():
    dir_path = "crates/bcinr-logic/src/algorithms/"
    for filename in os.listdir(dir_path):
        if filename.endswith(".rs"):
            path = os.path.join(dir_path, filename)
            with open(path, "r") as f:
                content = f.read()
            
            # Look for the malformed pattern
            # black_box(res)\n            \n})\n        });
            new_content = re.sub(
                r"black_box\(res\)\s*\n\s*\n}\)\n\s*\}\);",
                r"black_box(res)\n            })\n        });",
                content
            )
            
            if new_content != content:
                print(f"Fixing {filename}")
                with open(path, "w") as f:
                    f.write(new_content)

if __name__ == "__main__":
    fix_files()
