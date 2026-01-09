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
        let program = [
            0xE8, // 0x0000 INX
            0xC8, // 0x0001 INY
        ];
        m.load(0, &program).unwrap();
        m.run();
        assert_eq!(m.reg("XR"), 1, "wrong X value");
        assert_eq!(m.reg("YR"), 1, "wrong Y value");
    }

    #[test]
    fn machine_6502_loads_immediate_into_register_a() {
        let mut m = MOS6502::new();
        let program = [
            0xA9, 0xFF, // 0x0000 LDA #FF
        ];
        m.load(0, &program).unwrap();
        m.run();

        assert_eq!(m.reg("AC"), 0xFF, "wrong AC value");
    }

    #[test]
    fn lda_absolute_loads_value_from_given_address() {
        let mut m = MOS6502::new();

        let program = [
            0xAD, 0x04, 0x00, // 0x0000 LDA $0004
            0x00, //             0x0003 HLT
            0xFF, //             0x0004 DB #FF
        ];
        m.load(0, &program).unwrap();
        m.run();

        assert_eq!(m.reg("AC"), 0xFF, "wrong AC value");
    }

    #[test]
    fn clc_clears_carry_flag() {
        let mut m = MOS6502::new();
        let status = m.reg_mut("SR");
        *status |= CARRY;

        let program = [
            0x18, // 0x0000 CLC)
        ];
        m.load(0, &program).unwrap();
        m.run();

        assert_eq!(m.reg("SR") & CARRY, 0);
    }

    #[test]
    fn cld_clears_decimal_flag() {
        let mut m = MOS6502::new();
        let status = m.reg_mut("SR");
        *status |= DECIMAL;

        let program = [
            0xD8, // 0x0000 CLD
        ];
        m.load(0, &program).unwrap();
        m.run();

        assert_eq!(m.reg("SR") & DECIMAL, 0);
    }

    #[test]
    fn adc_adds_immediate_value_to_register_with_carry_and_sets_carry() {
        let mut m = MOS6502::new();

        let status = m.reg_mut("SR");
        *status |= CARRY;

        let program = [
            0xA9, 0xFE, // 0x0000 LDA #FE
            0x69, 0x01, // 0x0002 ADC #01
        ];
        m.load(0, &program).unwrap();
        m.run();

        assert_eq!(m.reg("AC"), 0x00);
        assert_eq!(m.reg("SR") & CARRY, 1);
    }

    #[test]
    fn adc_absolute_adds_value_in_address_to_register() {
        let mut m = MOS6502::new();

        let program = [
            0x6D, 0x04, 0x00, // 0x0000 ADC $0004
            0x00, //             0x0003 HLT
            0x01, //             0x0004 DB #01
        ];
        m.load(0, &program).unwrap();
        m.run();

        assert_eq!(m.reg("AC"), 0x01);
        assert_eq!(m.reg("SR") & CARRY, 0x00);
    }

    #[test]
    fn adc_absolute_adds_value_in_address_to_register_with_overflow() {
        let mut m = MOS6502::new();

        m.reg_set("AC", 0x01);

        let program = [
            0x6D, 0x04, 0x00, // 0x0000 ADC $0004
            0x00, //             0x0003 HLT
            0xFF, //             0x0004 DB #FF
        ];
        m.load(0, &program).unwrap();
        m.run();

        assert_eq!(m.reg("AC"), 0x00);
        assert_eq!(m.reg("SR") & CARRY, 0x01);
    }
}
