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
        mnemonic: "BRK",
        mode: Mode::Implied,
        opcode: 0x00,
        bytes: 1,
        cycles: 7,
        execute: |m| {
            m.exception = Some("break".into());
        },
        test: |m| {
            m.run_program(&[
                0x00, // 0x0000 BRK
            ]);
            assert_eq!(m.pc, 0x0001, "wrong PC");
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
        test: |m| {
            m.run_program(&[
                0x18, //       0x0000 CLC
                0xA9, 0xFE, // 0x0001 LDA #FE
                0x69, 0x01, // 0x0003 ADC #01
                0x00, //       0x0005 BRK
            ]);
            assert_eq!(m.reg("AC"), 0xFF, "wrong AC");
            assert!(!m.test_bit("SR", CARRY), "carry set");
            println!("{m}");
            m.run_program(&[
                //             ;A=FF, C=0
                0x69, 0x01, // 0x0000 ADC #01
                0x00, //       0x0002 BRK
            ]);
            assert_eq!(m.reg("AC"), 0x00, "wrong AC");
            assert!(m.test_bit("SR", CARRY), "carry clear after overflow");
        },
    },
    Instruction {
        mnemonic: "ADC",
        mode: Mode::Absolute,
        opcode: 0x6D,
        bytes: 3,
        cycles: 4,
        execute: |m| {
            let addr = m.get16(m.pc());
            m.advance(2);

            let operand = m.get8(addr);
            adc(m, operand);
        },
        test: |m| {
            m.run_program(&[
                0x18, //             0x0000 CLC
                0xA9, 0x01, //       0x0001 LDA #01
                0x6D, 0x07, 0x00, // 0x0003 ADC $0007
                0x00, //             0x0006 BRK
                0x01, //             0x0007 DB #01
            ]);
            assert_eq!(m.reg("AC"), 0x02, "wrong AC");
            assert!(!m.test_bit("SR", CARRY), "carry set");
            m.run_program(&[
                //                   ;A=02, C=0
                0x6D, 0x04, 0x00, // 0x0000 ADC $0004
                0x00, //             0x0003 BRK
                0xFF, //             0x0004 DB #FF
            ]);
            assert_eq!(m.reg("AC"), 0x1, "wrong AC");
            assert!(m.test_bit("SR", CARRY), "carry clear after overflow");
        },
    },
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
        test: |m| {
            m.set_bit("SR", CARRY);
            m.run_program(&[
                0x18, // 0x0000 CLC
            ]);
            assert!(!m.test_bit("SR", CARRY), "carry set");
        },
    },
    Instruction {
        mnemonic: "CLD",
        mode: Mode::Implied,
        opcode: 0xD8,
        bytes: 1,
        cycles: 2,
        execute: |m| {
            m.clear_bit("SR", DECIMAL);
        },
        test: |m| {
            m.set_bit("SR", DECIMAL);
            m.run_program(&[
                0xD8, // 0x0000 CLD
            ]);
            assert!(!m.test_bit("SR", DECIMAL), "decimal mode set");
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
        test: |m| {
            m.run_program(&[
                0xE8, // 0x0000 INX
            ]);
            assert_eq!(m.reg("XR"), 0x01, "wrong XR");
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
        test: |m| {
            m.run_program(&[
                0xC8, // 0x0000 INY
            ]);
            assert_eq!(m.reg("YR"), 0x01, "wrong YR");
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
            m.set_reg("AC", value);
        },
        test: |m| {
            m.run_program(&[
                0xA9, 0xFF, // 0x0000 LDA #FF
            ]);
            assert_eq!(m.reg("AC"), 0xFF, "wrong AC");
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
            m.advance(2);

            let value = m.get8(addr);
            m.set_reg("AC", value);
        },
        test: |m| {
            m.run_program(&[
                0xAD, 0x04, 0x00, // 0x0000 LDA $0004
                0x00, //             0x0003 BRK
                0xFF, //             0x0004 DB #FF
            ]);
            assert_eq!(m.reg("AC"), 0xFF, "wrong AC");
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
            m.advance(2);

            let value = m.reg("AC");
            m.set8(addr, value);
        },
        test: |m| {
            m.run_program(&[
                0xA9, 0x42, //       0x0000 LDA #42
                0x8D, 0x06, 0x00, // 0x0002 STA $0006
                0x00, //             0x0005 BRK
                0x00, //             0x0006 DB #0
            ]);
            assert_eq!(m.get8(0x0006), 0x42, "wrong value at address");
        },
    },
];
