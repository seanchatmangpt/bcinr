#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceEnvelope {
    pub cpu: u64,
    pub memory: u64,
    pub io: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Allocation {
    pub cpu: u64,
    pub memory: u64,
    pub io: u64,
}

impl ResourceEnvelope {
    pub const ZERO: Self = Self { cpu: 0, memory: 0, io: 0 };

    pub const fn admits(&self, allocation: &Allocation) -> bool {
        allocation.cpu <= self.cpu
            && allocation.memory <= self.memory
            && allocation.io <= self.io
    }

    /// Preserve the existing subtraction semantics: this returns the residual
    /// envelope after a bounded allocation, never an actuation grant.
    pub const fn child(&self, allocation: &Allocation) -> Option<Self> {
        if self.admits(allocation) {
            Some(Self {
                cpu: self.cpu - allocation.cpu,
                memory: self.memory - allocation.memory,
                io: self.io - allocation.io,
            })
        } else {
            None
        }
    }

    pub const fn granted(allocation: &Allocation) -> Self {
        Self {
            cpu: allocation.cpu,
            memory: allocation.memory,
            io: allocation.io,
        }
    }
}

/// Pure allocation receipt. It proves arithmetic conservation only; it does
/// not authorize or execute an effect.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BudgetReceipt {
    pub before: ResourceEnvelope,
    pub granted: ResourceEnvelope,
    pub after: ResourceEnvelope,
}

impl BudgetReceipt {
    pub fn conserves(&self) -> bool {
        self.after.cpu.checked_add(self.granted.cpu) == Some(self.before.cpu)
            && self.after.memory.checked_add(self.granted.memory) == Some(self.before.memory)
            && self.after.io.checked_add(self.granted.io) == Some(self.before.io)
    }

    /// A nested allocator receives exactly the resources granted by its
    /// parent; it cannot inherit the parent's residual budget.
    pub const fn child_ledger(&self) -> BudgetLedger {
        BudgetLedger::new(self.granted)
    }
}

/// Immutable, authority-free budget state for recursive SA2A allocation.
///
/// CMCA may decide an allocation vector, but this ledger only checks and
/// accounts for bounded resource conservation. No receipt emitted here is an
/// actuation certificate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BudgetLedger {
    pub capacity: ResourceEnvelope,
    pub remaining: ResourceEnvelope,
}

impl BudgetLedger {
    pub const fn new(capacity: ResourceEnvelope) -> Self {
        Self {
            capacity,
            remaining: capacity,
        }
    }

    pub const fn propose(&self, allocation: Allocation) -> Option<BudgetReceipt> {
        match self.remaining.child(&allocation) {
            Some(after) => Some(BudgetReceipt {
                before: self.remaining,
                granted: ResourceEnvelope::granted(&allocation),
                after,
            }),
            None => None,
        }
    }

    pub fn advance(&self, receipt: BudgetReceipt) -> Option<Self> {
        if receipt.before != self.remaining || !receipt.conserves() {
            return None;
        }
        Some(Self {
            capacity: self.capacity,
            remaining: receipt.after,
        })
    }
}
