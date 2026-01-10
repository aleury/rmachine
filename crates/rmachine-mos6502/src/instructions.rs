use rmachine_core::{machine::Mode, prelude::*};

use crate::flags::{CARRY, DECIMAL};

/// Performs ADC (Add with Carry) operation.
/// Adds the operand and carry flag to the accumulator, updating the carry flag on overflow.
fn adc(m: &mut Machine, operand: u8) {
    let carry = m.reg("SR") & CARRY;
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
}

pub const INSTRUCTIONS: &[Instruction] = &[
    Instruction {
        mnemonic: "CLC",
        mode: Mode::Implied,
        opcode: 0x18,
        bytes: 1,
        cycles: 2,
        execute: |m| {
            let status = m.reg_mut("SR");
            *status &= !CARRY;
        },
    },
    Instruction {
        mnemonic: "ADC",
        mode: Mode::Immediate,
        opcode: 0x69,
        bytes: 2,
        cycles: 2,
        execute: |m| {
            let operand = m.fetch();
            adc(m, operand);
        },
    },
    Instruction {
        mnemonic: "ADC",
        mode: Mode::Absolute,
        opcode: 0x6D,
        bytes: 2,
        cycles: 2,
        execute: |m| {
            let addr = m.get16(m.pc());
            let operand = m.get8(addr);
            adc(m, operand);
        },
    },
    Instruction {
        mnemonic: "STA",
        mode: Mode::Absolute,
        opcode: 0x8D,
        bytes: 3,
        cycles: 4,
        execute: |m| {
            let addr = m.get16(m.pc());
            let value = m.reg("AC");
            m.set8(addr, value);
        },
    },
    Instruction {
        mnemonic: "LDA",
        mode: Mode::Immediate,
        opcode: 0xA9,
        bytes: 2,
        cycles: 2,
        execute: |m| {
            let value = m.fetch();
            m.reg_set("AC", value);
        },
    },
    Instruction {
        mnemonic: "LDA",
        mode: Mode::Absolute,
        opcode: 0xAD,
        bytes: 3,
        cycles: 2,
        execute: |m| {
            let addr = m.get16(m.pc());
            let value = m.get8(addr);
            m.reg_set("AC", value);
        },
    },
    Instruction {
        mnemonic: "INY",
        mode: Mode::Implied,
        opcode: 0xC8,
        bytes: 1,
        cycles: 2,
        execute: |m| {
            let reg = m.reg_mut("YR");
            *reg = reg.wrapping_add(1);
        },
    },
    Instruction {
        mnemonic: "CLD",
        mode: Mode::Implied,
        opcode: 0xD8,
        bytes: 1,
        cycles: 2,
        execute: |m| {
            let status = m.reg_mut("SR");
            *status &= !DECIMAL;
        },
    },
    Instruction {
        mnemonic: "INX",
        mode: Mode::Implied,
        opcode: 0xE8,
        bytes: 1,
        cycles: 2,
        execute: |m| {
            let reg = m.reg_mut("XR");
            *reg = reg.wrapping_add(1);
        },
    },
];
