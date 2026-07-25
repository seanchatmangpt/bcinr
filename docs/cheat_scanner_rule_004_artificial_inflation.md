Here is the documentation on how the cheat scanner detects `CHEAT-004` (Artificial file inflation) based on `tools/bcinr-cheat-scanner/src/main.rs`.

The detection is implemented in the `scan_file_text_rules` function. It operates on the raw text of the file (`src`) and checks for two distinct patterns:

### 1. Sentinel Padding String
It searches the entire file text for an exact sentinel string:
```rust
if src.contains("PADDING ENSURING FILE LENGTH REQUIREMENT") { ... }
```
If this specific string is found anywhere in the file, it flags the file with `"artificial file-length inflation detected"`.

### 2. Numbered Padding Comment Blocks
It iterates through the file line-by-line and maintains a `consecutive_padding` counter to identify repetitive numbered comments:
```rust
let mut consecutive_padding = 0;
for line in src.lines() {
    if line.trim().starts_with("//") {
        let after_slashes = line.trim()[2..].trim();
        if after_slashes.contains(". Line") {
            consecutive_padding += 1;
            if consecutive_padding >= 5 {
                // ... flags as "numbered padding block detected"
                break;
            }
        }
    } else {
        consecutive_padding = 0;
    }
}
```
**How this loop works:**
- It checks if a line (after trimming leading and trailing whitespace) starts with `//`.
- If it does, it strips the `//` and trims the remaining text.
- If the remaining text contains the string `". Line"`, the `consecutive_padding` counter is incremented.
- **Important Nuance:** If a line is a comment but *does not* contain `". Line"`, the counter is neither incremented nor reset (it ignores standard comments interspersed within the block).
- If the line is *not* a comment (i.e., it's actual code or an empty line), the `else` block triggers and resets the counter to `0`.
- If the counter reaches `5` (meaning there are 5 `. Line` comments within a contiguous block of comments without intervening code), the scanner flags it as `"numbered padding block detected"` and stops searching that file.
