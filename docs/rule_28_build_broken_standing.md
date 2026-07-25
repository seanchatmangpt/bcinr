Here is the information regarding the `BUILD_BROKEN` standing based on `AGENTS.md`:

In `AGENTS.md`, **Rule 28: Standing vocabulary** establishes a bounded set of strict labels used to define the exact status and evidence standing of the project.

The `BUILD_BROKEN` standing specifically means:
> **The pinned build fails.**

### Conditions for Assignment
This standing is assigned to the project or a component under the following conditions:
- **Build Failure:** The implementation cannot be successfully compiled or built within the pinned target environment.
- **Dependency Limitations:** The rule dictates that *"Claims may not exceed their weakest load-bearing dependency."* This means that if any underlying dependency or required component fails to build, the higher-level feature or project inherits the `BUILD_BROKEN` standing or is restricted by that weakest link's standing.
