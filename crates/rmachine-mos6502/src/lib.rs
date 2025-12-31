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
                memory_size: 1024,
                registers: &["SR", "AC", "XR", "YR", "SP"],
                instructions: INSTRUCTIONS,
                frequency_mhz: 2.0,
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
    use crate::flags::{CARRY, DECIMAL};

    use super::*;

    #[test]
    fn machine_6502_increments_registers_x_and_y() {
        let mut m = MOS6502::new();
        let program = [m.opcode("INX"), m.opcode("INY")];
        m.load(0, &program).unwrap();
        m.run();
        assert_eq!(m.reg("XR"), 1, "wrong X value");
        assert_eq!(m.reg("YR"), 1, "wrong Y value");
    }

    #[test]
    fn machine_6502_loads_immediate_into_register_a() {
        let mut m = MOS6502::new();
        let program = [m.opcode("LDA"), 0xFF];
        m.load(0, &program).unwrap();
        m.run();

        assert_eq!(m.reg("AC"), 0xFF, "wrong AC value");
    }

    #[test]
    fn clc_clears_carry_flag() {
        let mut m = MOS6502::new();
        let status = m.reg_mut("SR");
        *status |= CARRY;

        let program = [m.opcode("CLC")];
        m.load(0, &program).unwrap();
        m.run();

        assert_eq!(m.reg("SR") & CARRY, 0);
    }

    #[test]
    fn cld_clears_decimal_flag() {
        let mut m = MOS6502::new();
        let status = m.reg_mut("SR");
        *status |= DECIMAL;

        let program = [m.opcode("CLD")];
        m.load(0, &program).unwrap();
        m.run();

        assert_eq!(m.reg("SR") & DECIMAL, 0);
    }

    #[test]
    fn adc_adds_to_register_with_carry_and_sets_carry() {
        let mut m = MOS6502::new();

        let status = m.reg_mut("SR");
        *status |= CARRY;

        let program = [m.opcode("LDA"), 0xFE, m.opcode("ADC"), 0x01];
        m.load(0, &program).unwrap();
        m.run();

        assert_eq!(m.reg("AC"), 0x00);
        assert_eq!(m.reg("SR") & CARRY, 1);
    }
}
