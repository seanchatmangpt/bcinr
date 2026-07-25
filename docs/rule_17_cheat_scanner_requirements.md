Based on Rule 17 ("Cheat-scanner requirements") in `AGENTS.md`, here are the specific requirements for the `bcinr-cheat-scanner`:

### What it must inspect
The scanner is required to perform comprehensive source and syntax verification, specifically it must:
* Parse the full syntax tree
* Scan both public and private functions
* Inspect macro definitions as well as their expanded output
* Scan generated Rust code
* Normalize whitespace and comments (where required)
* Strip numeric separators
* Detect equivalent hex spellings
* Inspect test references and benchmark targets

### What it must report
When the scanner identifies an issue, it must report the **exact file, span, and rule identifier**. 

The finding must be formatted using the following structure:
```text
CHEAT[rule-id]
```
*Example:* `CHEAT[CHEAT-006]: prohibited operator hidden in macro expansion`

### What happens when a finding is emitted
* **Blocks merge:** Every single finding emitted by the cheat scanner strictly blocks the code from being merged.
* **No silent suppressions:** Baseline suppressions are explicitly prohibited unless accompanied by a separately admitted waiver artifact.
