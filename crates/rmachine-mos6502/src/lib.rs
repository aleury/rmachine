use std::ops::{Deref, DerefMut};

use rmachine_core::prelude::*;

mod flags;
mod instructions;

#[derive(Debug)]
pub struct MOS6502(Machine);

impl MOS6502 {
    /// Constructs a new 6502.
    ///
    /// # Panics
    ///
    /// Panics if any duplicate opcodes are defined.
    #[must_use]
    pub fn new() -> Self {
        MOS6502::default()
    }
}

impl Default for MOS6502 {
    /// Constructs a new 6502.
    ///
    /// # Panics
    ///
    /// Panics if any duplicate opcodes are defined.
    fn default() -> Self {
        use instructions::INSTRUCTIONS;
        Self(
            MachineBuilder {
                memory_size: 0x10_000, // 64KiB
                registers: &["SR", "AC", "XR", "YR", "SP"],
                instructions: INSTRUCTIONS,
                frequency_hz: 2_000_000,
            }
            .build(),
        )
    }
}

impl Deref for MOS6502 {
    type Target = Machine;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MOS6502 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn machine_self_tests_pass() {
        MOS6502::new().self_test();
    }
}
