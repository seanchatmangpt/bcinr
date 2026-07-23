//! Build broker — resource coordination for heavy commands.
//!
//! Heavy builds (cargo build, wasm-pack, tsc, gradle) must be acquired through
//! the broker. Direct heavy builds are DIRECT_HEAVY_COMMAND_BLOCKED.
//! Each slot acquisition emits OCEL. Each denial emits OCEL.

use serde::{Deserialize, Serialize};

pub const MAX_HEAVY_SLOTS: usize = 1;

/// Categories of commands that require broker admission.
pub const HEAVY_COMMANDS: &[&str] = &[
    "cargo build",
    "cargo test",
    "cargo check",
    "wasm-pack build",
    "npm run build",
    "tsc",
    "gradle build",
    "make",
    "just build",
    "just test",
    "pnpm build",
    "pnpm test",
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BrokerSlotStatus {
    /// No slot requested yet.
    Idle,
    /// Slot available, not yet acquired.
    Available,
    /// Slot acquired — heavy build in progress.
    Acquired,
    /// Slot denied — another build is active.
    Denied,
    /// Slot released after build complete.
    Released,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildBrokerState {
    pub slot_status: BrokerSlotStatus,
    pub active_build: Option<String>,
    pub queued_count: usize,
    pub max_slots: usize,
    pub last_ocel_event: Option<String>,
    pub denial_count: usize,
}

impl Default for BuildBrokerState {
    fn default() -> Self {
        Self {
            slot_status: BrokerSlotStatus::Idle,
            active_build: None,
            queued_count: 0,
            max_slots: MAX_HEAVY_SLOTS,
            last_ocel_event: None,
            denial_count: 0,
        }
    }
}

impl BuildBrokerState {
    pub fn can_acquire(&self) -> bool {
        !matches!(self.slot_status, BrokerSlotStatus::Acquired)
    }

    pub fn request_slot(&mut self, command: &str) -> Result<(), BrokerDenial> {
        if self.slot_status == BrokerSlotStatus::Acquired {
            self.denial_count += 1;
            self.last_ocel_event = Some(format!("BUILD_SLOT_DENIED:{command}"));
            return Err(BrokerDenial {
                command: command.to_string(),
                reason: format!("Build slot occupied by: {:?}", self.active_build),
            });
        }
        self.slot_status = BrokerSlotStatus::Available;
        self.last_ocel_event = Some(format!("BUILD_SLOT_AVAILABLE:{command}"));
        Ok(())
    }

    pub fn acquire_slot(&mut self, command: &str) -> Result<(), BrokerDenial> {
        if self.slot_status != BrokerSlotStatus::Available {
            return Err(BrokerDenial {
                command: command.to_string(),
                reason: "Slot not available — call request_slot first".into(),
            });
        }
        self.slot_status = BrokerSlotStatus::Acquired;
        self.active_build = Some(command.to_string());
        self.last_ocel_event = Some(format!("BUILD_SLOT_ACQUIRED:{command}"));
        Ok(())
    }

    pub fn release_slot(&mut self) {
        let cmd = self.active_build.take().unwrap_or_default();
        self.slot_status = BrokerSlotStatus::Released;
        self.last_ocel_event = Some(format!("BUILD_SLOT_RELEASED:{cmd}"));
    }

    pub fn status_label(&self) -> &'static str {
        match self.slot_status {
            BrokerSlotStatus::Idle => "IDLE",
            BrokerSlotStatus::Available => "AVAILABLE",
            BrokerSlotStatus::Acquired => "ACQUIRED",
            BrokerSlotStatus::Denied => "DENIED",
            BrokerSlotStatus::Released => "RELEASED",
        }
    }
}

#[derive(Debug, Clone)]
pub struct BrokerDenial {
    pub command: String,
    pub reason: String,
}

impl std::fmt::Display for BrokerDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BUILD_SLOT_DENIED: {} ({})", self.command, self.reason)
    }
}

/// Check whether a command string is a heavy command requiring broker admission.
pub fn is_heavy_command(cmd: &str) -> bool {
    let cmd_lower = cmd.to_lowercase();
    HEAVY_COMMANDS.iter().any(|&h| cmd_lower.contains(h))
}

/// Check for direct heavy command execution (bypassing broker).
/// Returns Some(diagnostic) if the command should be blocked.
pub fn check_direct_command(cmd: &str, state: &BuildBrokerState) -> Option<DirectCommandViolation> {
    if !is_heavy_command(cmd) {
        return None;
    }
    if state.slot_status == BrokerSlotStatus::Acquired {
        return None;
    }
    Some(DirectCommandViolation {
        command: cmd.to_string(),
    })
}

#[derive(Debug, Clone)]
pub struct DirectCommandViolation {
    pub command: String,
}

impl DirectCommandViolation {
    pub fn diagnostic_code(&self) -> &'static str {
        "DIRECT_HEAVY_COMMAND_BLOCKED"
    }

    pub fn message(&self) -> String {
        format!(
            "Direct heavy command '{}' blocked. Acquire a build slot first via bcinrPddl.requestBuildSlot.",
            self.command
        )
    }
}
