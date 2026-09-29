use serde::{Deserialize, Serialize};

use super::SecurityRefusal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceEnvelope {
    pub compute_units: u64,
    pub io_bytes: u64,
    pub effects: u64,
}

impl ResourceEnvelope {
    pub fn validate(self) -> Result<Self, SecurityRefusal> {
        if self.compute_units == 0 || self.effects == 0 {
            return Err(SecurityRefusal::InvalidResourceEnvelope);
        }
        Ok(self)
    }

    #[must_use]
    pub const fn contains(self, child: Self) -> bool {
        child.compute_units <= self.compute_units
            && child.io_bytes <= self.io_bytes
            && child.effects <= self.effects
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetLedger {
    root: ResourceEnvelope,
    committed: ResourceEnvelope,
}

impl BudgetLedger {
    pub fn new(root: ResourceEnvelope) -> Result<Self, SecurityRefusal> {
        Ok(Self {
            root: root.validate()?,
            committed: ResourceEnvelope { compute_units: 0, io_bytes: 0, effects: 0 },
        })
    }

    pub fn allocate(&mut self, child: ResourceEnvelope) -> Result<(), SecurityRefusal> {
        child.validate()?;
        let next = ResourceEnvelope {
            compute_units: self.committed.compute_units.checked_add(child.compute_units).ok_or(SecurityRefusal::ResourceAmplification)?,
            io_bytes: self.committed.io_bytes.checked_add(child.io_bytes).ok_or(SecurityRefusal::ResourceAmplification)?,
            effects: self.committed.effects.checked_add(child.effects).ok_or(SecurityRefusal::ResourceAmplification)?,
        };
        if !self.root.contains(next) {
            return Err(SecurityRefusal::ResourceAmplification);
        }
        self.committed = next;
        Ok(())
    }

    #[must_use]
    pub const fn committed(&self) -> ResourceEnvelope {
        self.committed
    }
}
