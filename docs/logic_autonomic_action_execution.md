Here is the documentation regarding `AutonomicAction` based on the codebase search in `/Users/sac/bcinr/crates/`:

I found the definitions in `crates/bcinr-logic/src/autonomic/kernel.rs` and the execution implementation in `crates/bcinr-logic/src/models/vision_2030.rs`.

Interestingly, `AutonomicAction` **does** utilize standard Rust enums for its data representation (`ActionKind` and `ActionRisk`). However, the *execution logic* strictly avoids enum-based branching (like `match` or `if/else`) to fully comply with the project's **CC=1 Radon Law**. 

Here is how actions are defined and processed branchlessly:

### 1. Action Definition (`kernel.rs`)
The structures use basic enums but derive `PartialEq` and `PartialOrd`. This allows them to be evaluated in boolean expressions and cast directly to integers for masking.

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActionKind {
    Repair,
    Optimize,
    Scale,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum ActionRisk {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct AutonomicAction {
    pub id: u32,
    pub kind: ActionKind,
    pub risk: ActionRisk,
    pub description: String,
}
```

### 2. Branchless Acceptance (`vision_2030.rs`)
Rather than pattern matching on risk levels, risk evaluation relies on integer casting and mathematical comparisons:

```rust
fn accept(&self, action: &AutonomicAction, _state: &AutonomicState) -> bool {
    (action.risk as u8) < (ActionRisk::Critical as u8)
}
```

### 3. Mask-Based Execution (`vision_2030.rs`)
During execution, action types are converted into bitmasks and index selections. This forces a constant-time, fixed instruction shape state transition without ever branching:

```rust
fn execute(&mut self, action: AutonomicAction) -> AutonomicResult {
    // 1. Evaluate enum equality and cast directly to an integer (0 or 1)
    let is_repair = (action.kind == ActionKind::Repair) as u64;
    
    // 2. Generate a full-width mask (0x0000000000000000 or 0xFFFFFFFFFFFFFFFF)
    let mask = 0u64.wrapping_sub(is_repair);
    
    let mut reset = KBitSet::<WORDS>::zero();
    reset.set(0);
    
    // 3. Branchless state transition using SWAR/bitwise selection
    // Equation: (new_state & mask) | (current_state & ~mask)
    self.marking.current.words[0] =
        (reset.words[0] & mask) | (self.marking.current.words[0] & !mask);
        
    // 4. Update booleans utilizing logical operations rather than conditional blocks
    self.state.drift_detected = self.state.drift_detected && is_repair == 0;
    
    // 5. Array-based index selection for non-bitwise types like f32
    self.state.integrity = [self.state.integrity, 1.0][is_repair as usize];
    
    AutonomicResult {
        success: true,
        latency_cycles: 100,
        manifest_hash: 0xABC,
    }
}
```

Through mathematical casting, full-width masking via `0u64.wrapping_sub()`, array indexing, and boolean bitwise operators, the substrate preserves its deterministic properties seamlessly.
