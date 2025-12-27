use std::ops::{Deref, DerefMut};

use rmachine_core::prelude::*;

const REGISTERS: &[&str] = &["SR", "AC", "XR", "YR", "SP"];

const CARRY: u8 = 0b0000_0001;
const DECIMAL: u8 = 0b0000_1000;

const INSTRUCTIONS: &[Instruction] = &[
    Instruction {
        mnemonic: "CLC",
        opcode: 0x18,
        operands: Operands::Zero,
        execute: |m| {
            let status = m.reg_mut("SR");
            *status &= !CARRY;
        },
    },
    Instruction {
        mnemonic: "CLD",
        opcode: 0xD8,
        operands: Operands::Zero,
        execute: |m| {
            let status = m.reg_mut("SR");
            *status &= !DECIMAL;
        },
    },
    Instruction {
        mnemonic: "ADC",
        opcode: 0x69,
        operands: Operands::One,
        execute: |m| {
            let carry = m.reg("SR") & CARRY;
            let operand = m.fetch().unwrap();
            let reg = m.reg_mut("AC");

            let (result, overflow) = reg.overflowing_add(operand);
            let (result, overflow2) = result.overflowing_add(carry);
            *reg = result;

            let status = m.reg_mut("SR");
            if overflow || overflow2 {
                *status |= CARRY;
            } else {
                *status &= !CARRY;
            }
        },
    },
    Instruction {
        mnemonic: "LDA",
        opcode: 0xA9,
        operands: Operands::One,
        execute: |m| {
            let value = m.fetch().unwrap();
            m.reg_set("AC", value);
        },
    },
    Instruction {
        mnemonic: "INY",
        opcode: 0xC8,
        operands: Operands::Zero,
        execute: |m| {
            let reg = m.reg_mut("YR");
            *reg = reg.wrapping_add(1);
        },
    },
    Instruction {
        mnemonic: "INX",
        opcode: 0xE8,
        operands: Operands::Zero,
        execute: |m| {
            let reg = m.reg_mut("XR");
            *reg = reg.wrapping_add(1);
        },
    },
];

#[derive(Debug)]
pub struct MOS6502(Machine);

impl MOS6502 {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
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

impl Default for MOS6502 {
    fn default() -> Self {
        let machine = MachineBuilder::default()
            .with_memory(1024)
            .with_registers(REGISTERS)
            .with_instructions(INSTRUCTIONS)
            .build();
        Self(machine)
    }
}

#[cfg(test)]
mod tests {
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
