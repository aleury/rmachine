use std::ops::{Deref, DerefMut};

use rmachine_core::prelude::*;

mod constants;
mod flags;
mod handlers;
mod instructions;

pub use constants::*;

pub const MEMORY_SIZE: usize = 0x10_000; // 64KiB

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
        use handlers::HANDLERS;
        use instructions::INSTRUCTIONS;
        Self(
            MachineBuilder {
                registers: &["SR", "AC", "XR", "YR", "SP"],
                instructions: INSTRUCTIONS,
                handlers: (*HANDLERS).clone(),
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

    fn new_memory() -> Memory {
        Memory::new(MEMORY_SIZE)
    }

    #[test]
    fn machine_self_tests_pass() {
        MOS6502::new().self_test();
    }

    #[test]
    fn machine_jumps_to_reset_vector_on_reset() {
        let mut m = MOS6502::new();
        let mut mem = new_memory();
        mem.set16(RESET_VECTOR, 0x1000);
        m.signal(&mut mem, RESET);
        assert_eq!(m.pc, 0x1001, "PC {:#06X} != 0x1001", m.pc);
    }

    #[test]
    fn machine_jumps_to_irq_vector_on_irq() {
        let mut m = MOS6502::new();
        let mut mem = new_memory();
        mem.set16(IRQ_VECTOR, 0x1000);
        m.signal(&mut mem, IRQ);
        assert_eq!(m.pc, 0x1001, "PC {:#06X} != 0x1001", m.pc);
    }

    #[test]
    fn machine_jumps_to_nmi_vector_on_nmi() {
        let mut m = MOS6502::new();
        let mut mem = new_memory();
        mem.set16(NMI_VECTOR, 0x1000);
        m.signal(&mut mem, NMI);
        assert_eq!(m.pc, 0x1001, "PC {:#06X} != 0x1001", m.pc);
    }
}
