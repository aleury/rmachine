use rmachine_core::prelude::*;

use crate::flags::{CARRY, DECIMAL};

pub const INSTRUCTIONS: &[Instruction] = &[
    Instruction {
        mnemonic: "CLC",
        opcode: 0x18,
        operands: Operands::Zero,
        execute: |m| {
            let status = m.reg_mut("SR");
            *status &= !CARRY;
        },
        cycles: 2,
    },
    Instruction {
        mnemonic: "CLD",
        opcode: 0xD8,
        operands: Operands::Zero,
        execute: |m| {
            let status = m.reg_mut("SR");
            *status &= !DECIMAL;
        },
        cycles: 2,
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
        cycles: 2,
    },
    Instruction {
        mnemonic: "LDA",
        opcode: 0xA9,
        operands: Operands::One,
        execute: |m| {
            let value = m.fetch().unwrap();
            m.reg_set("AC", value);
        },
        cycles: 2,
    },
    Instruction {
        mnemonic: "INY",
        opcode: 0xC8,
        operands: Operands::Zero,
        execute: |m| {
            let reg = m.reg_mut("YR");
            *reg = reg.wrapping_add(1);
        },
        cycles: 2,
    },
    Instruction {
        mnemonic: "INX",
        opcode: 0xE8,
        operands: Operands::Zero,
        execute: |m| {
            let reg = m.reg_mut("XR");
            *reg = reg.wrapping_add(1);
        },
        cycles: 2,
    },
];
