use rmachine_core::{exception::Exception, machine::Mode, prelude::*};

use crate::flags::{CARRY, DECIMAL, ZERO};

/// Performs ADC (Add with Carry) operation.
///
/// Adds the operand and carry flag to the accumulator, updating the carry flag.
fn adc(m: &mut Machine, operand: u8) {
    let carry = m.reg("SR") & CARRY;
    let reg = m.reg("AC");

    let (result, overflow) = reg.overflowing_add(operand);
    let (result, overflow2) = result.overflowing_add(carry);

    m.set_reg("AC", result);

    if overflow || overflow2 {
        m.set_bit("SR", CARRY);
    } else {
        m.clear_bit("SR", CARRY);
    }
}

pub const INSTRUCTIONS: &[Instruction] = &[
    Instruction {
        mnemonic: "ADC",
        mode: Mode::Immediate,
        opcode: 0x69,
        bytes: 2,
        cycles: 2,
        execute: |m| {
            let op = m.fetch8();
            adc(m, op);
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
            let addr = m.fetch16();
            let op = m.get8(addr);
            adc(m, op);
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
        mnemonic: "BRK",
        mode: Mode::Implied,
        opcode: 0x00,
        bytes: 1,
        cycles: 7,
        execute: |m| {
            m.trap(Exception::Break);
        },
        test: |m| {
            m.run_program(&[
                0x00, // 0x0000 BRK
            ]);
            assert_eq!(m.pc, 0x0001, "wrong PC");
        },
    },
    Instruction {
        mnemonic: "BNE",
        mode: Mode::Relative,
        opcode: 0xD0,
        bytes: 2,
        cycles: 2,
        execute: |m| {
            let offset = m.fetch8();
            let signed_offset = offset.cast_signed();
            if !m.test_bit("SR", ZERO) {
                m.pc = m.pc.wrapping_add_signed(i16::from(signed_offset));
            }
        },
        test: |m| {
            // Test branch forward
            m.clear_bit("SR", ZERO);
            m.set_bit("SR", CARRY);
            m.run_program(&[
                0xD0, 0x01, // 0x0000 BNE $01
                0x00, //       0x0002 BRK (skipped)
                0x18, //       0x0003 CLC
                0x00, //       0x0004 BRK
            ]);
            assert_eq!(m.pc(), 0x0005, "wrong PC");
            assert!(!m.test_bit("SR", CARRY), "carry not cleared");

            // Test branch backward
            m.clear_bit("SR", ZERO);
            m.set_bit("SR", CARRY);
            m.run_program(&[
                0x4C, 0x05, 0x00, // $0000 JMP $0005
                0x18, //             $0003 CLC
                0x00, //             $0004 BRK
                0xD0, 0xFC, //       $0005 BNE $FC (branch to $0003, -4 back from pc := $0007)
                0x00, //             $0007 BRK (skipped)
            ]);
            assert_eq!(m.pc(), 0x0005, "wrong PC");
            assert!(!m.test_bit("SR", CARRY), "carry not cleared");
        },
    },
    Instruction {
        mnemonic: "CLC",
        mode: Mode::Implied,
        opcode: 0x18,
        bytes: 1,
        cycles: 2,
        execute: |m| m.clear_bit("SR", CARRY),
        test: |m| {
            m.set_bit("SR", CARRY);
            m.run_program(&[
                0x18, // 0x0000 CLC
                0x00, // 0x0001 BRK
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
        execute: |m| m.clear_bit("SR", DECIMAL),
        test: |m| {
            m.set_bit("SR", DECIMAL);
            m.run_program(&[
                0xD8, // 0x0000 CLD
                0x00, // 0x0001 BRK
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
            let result = m.reg("XR").wrapping_add(1);
            m.set_reg("XR", result);
        },
        test: |m| {
            m.run_program(&[
                0xE8, // 0x0000 INX
                0x00, // 0x0001 BRK
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
            let result = m.reg("YR").wrapping_add(1);
            m.set_reg("YR", result);
        },
        test: |m| {
            m.run_program(&[
                0xC8, // 0x0000 INY
                0x00, // 0x0001 BRK
            ]);
            assert_eq!(m.reg("YR"), 0x01, "wrong YR");
        },
    },
    Instruction {
        mnemonic: "JMP",
        mode: Mode::Absolute,
        opcode: 0x4C,
        bytes: 3,
        cycles: 3,
        execute: |m| m.pc = m.fetch16(),
        test: |m| {
            m.run_program(&[
                0x4C, 0x04, 0x00, // 0x0000 JMP $0004
                0x00, //             0x0003 BRK
                0x00, //             0x0004 BRK
            ]);
            assert_eq!(m.pc, 0x0005, "wrong PC");
        },
    },
    Instruction {
        mnemonic: "LDA",
        mode: Mode::Immediate,
        opcode: 0xA9,
        bytes: 2,
        cycles: 2,
        execute: |m| {
            let op = m.fetch8();
            m.set_reg("AC", op);
        },
        test: |m| {
            m.run_program(&[
                0xA9, 0xFF, // 0x0000 LDA #$FF
                0x00, //       0x0002 BRK
            ]);
            assert_eq!(m.reg("AC"), 0xFF, "wrong AC");
        },
    },
    Instruction {
        mnemonic: "LDA",
        mode: Mode::ZeroPage,
        opcode: 0xA5,
        bytes: 2,
        cycles: 3,
        execute: |m| {
            let addr = m.fetch8();
            let op = m.get8(u16::from(addr));
            m.set_reg("AC", op);
        },
        test: |m| {
            m.run_program(&[
                0xA5, 0x03, // 0x0000 LDA $03
                0x00, //       0x0002 BRK
                0xFF, //       0x0003 DB #$FF
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
            let addr = m.fetch16();
            let op = m.get8(addr);
            m.set_reg("AC", op);
        },
        test: |m| {
            m.run_program(&[
                0xAD, 0x04, 0x00, // 0x0000 LDA $0004
                0x00, //             0x0003 BRK
                0xFF, //             0x0004 DB #$FF
            ]);
            assert_eq!(m.reg("AC"), 0xFF, "wrong AC");
        },
    },
    Instruction {
        mnemonic: "LDX",
        mode: Mode::Immediate,
        opcode: 0xA2,
        bytes: 2,
        cycles: 2,
        execute: |m| {
            let op = m.fetch8();
            m.set_reg("XR", op);
        },
        test: |m| {
            m.run_program(&[
                0xA2, 0x2a, // 0x0000 LDX #$2a
                0x00, //       0x0002 BRK
            ]);
            assert_eq!(m.reg("XR"), 0x2a, "wrong XR");
        },
    },
    Instruction {
        mnemonic: "LDX",
        mode: Mode::ZeroPage,
        opcode: 0xA6,
        bytes: 2,
        cycles: 3,
        execute: |m| {
            let addr = m.fetch8();
            let op = m.get8(u16::from(addr));
            m.set_reg("XR", op);
        },
        test: |m| {
            m.run_program(&[
                0xA6, 0x03, // 0x0000 LDX $03
                0x00, //       0x0002 BRK
                0x2a, //       0x0003 DB #$2a
            ]);
            assert_eq!(m.reg("XR"), 0x2a, "wrong XR");
        },
    },
    Instruction {
        mnemonic: "LDX",
        mode: Mode::Absolute,
        opcode: 0xAE,
        bytes: 3,
        cycles: 4,
        execute: |m| {
            let addr = m.fetch16();
            let op = m.get8(addr);
            m.set_reg("XR", op);
        },
        test: |m| {
            m.run_program(&[
                0xAE, 0x04, 0x00, // 0x0000 LDX $0004
                0x00, //             0x0003 BRK
                0x2a, //             0x0004 DB #$2a
            ]);
            assert_eq!(m.reg("XR"), 0x2a, "wrong XR");
        },
    },
    Instruction {
        mnemonic: "LDY",
        mode: Mode::Immediate,
        opcode: 0xA0,
        bytes: 2,
        cycles: 2,
        execute: |m| {
            let op = m.fetch8();
            m.set_reg("YR", op);
        },
        test: |m| {
            m.run_program(&[
                0xA0, 0x2a, // 0x0000 LDY #$2a
                0x00, //       0x0002 BRK
            ]);
            assert_eq!(m.reg("YR"), 0x2a, "wrong YR");
        },
    },
    Instruction {
        mnemonic: "LDY",
        mode: Mode::ZeroPage,
        opcode: 0xA4,
        bytes: 2,
        cycles: 3,
        execute: |m| {
            let addr = m.fetch8();
            let op = m.get8(u16::from(addr));
            m.set_reg("YR", op);
        },
        test: |m| {
            m.run_program(&[
                0xA4, 0x03, // 0x0000 LDY $03
                0x00, //       0x0002 BRK
                0x2a, //       0x0003 DB #$2a
            ]);
            assert_eq!(m.reg("YR"), 0x2a, "wrong YR");
        },
    },
    Instruction {
        mnemonic: "LDY",
        mode: Mode::Absolute,
        opcode: 0xAC,
        bytes: 3,
        cycles: 4,
        execute: |m| {
            let addr = m.fetch16();
            let op = m.get8(addr);
            m.set_reg("YR", op);
        },
        test: |m| {
            m.run_program(&[
                0xAC, 0x04, 0x00, // 0x0000 LDY $0004
                0x00, //             0x0003 BRK
                0x2a, //             0x0004 DB #$2a
            ]);
            assert_eq!(m.reg("YR"), 0x2a, "wrong YR");
        },
    },
    Instruction {
        mnemonic: "STA",
        mode: Mode::Absolute,
        opcode: 0x8D,
        bytes: 3,
        cycles: 4,
        execute: |m| {
            let addr = m.fetch16();
            let op = m.reg("AC");
            m.set8(addr, op);
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
