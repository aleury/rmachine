use rmachine_core::{exception::Exception, machine::Mode, prelude::*};

use crate::flags::{CARRY, DECIMAL, INTERRUPT, NEGATIVE, OVERFLOW, ZERO};

/// Performs ADC (Add with Carry) operation.
///
/// Adds the operand and carry flag to the accumulator, updating the carry flag.
fn adc(m: &mut Machine, operand: u8) {
    let carry = m.reg("SR") & CARRY;
    let reg = m.reg("AC");

    let (result, overflow) = reg.overflowing_add(operand);
    let (result, overflow2) = result.overflowing_add(carry);

    m.set_reg("AC", result);

    update_carry_flag(m, overflow || overflow2);
    update_zero_flag(m, result);
}

fn dec(m: &mut Machine, addr: u16) {
    let result = m.get8(addr).wrapping_sub(1);
    m.set8(addr, result);
    update_zero_flag(m, result);
}

fn inc(m: &mut Machine, addr: u16) {
    let result = m.get8(addr).wrapping_add(1);
    m.set8(addr, result);
    update_zero_flag(m, result);
}

fn dec_reg(m: &mut Machine, reg: &'static str) {
    let result = m.reg(reg).wrapping_sub(1);
    m.set_reg(reg, result);
    update_zero_flag(m, result);
}

fn inc_reg(m: &mut Machine, reg: &'static str) {
    let result = m.reg(reg).wrapping_add(1);
    m.set_reg(reg, result);
    update_zero_flag(m, result);
}

fn load_reg(m: &mut Machine, reg: &'static str, value: u8) {
    m.set_reg(reg, value);
    update_zero_flag(m, value);
    update_negative_flag(m, value);
}

fn compare(m: &mut Machine, reg: &'static str, value: u8) {
    let reg_value = m.reg(reg);
    let result = reg_value.wrapping_sub(value);
    update_carry_flag(m, reg_value >= value);
    update_zero_flag(m, result);
    update_negative_flag(m, result);
}

fn update_zero_flag(m: &mut Machine, value: u8) {
    if value == 0 {
        m.set_bit("SR", ZERO);
    } else {
        m.clear_bit("SR", ZERO);
    }
}

fn update_negative_flag(m: &mut Machine, value: u8) {
    if value & NEGATIVE != 0 {
        m.set_bit("SR", NEGATIVE);
    } else {
        m.clear_bit("SR", NEGATIVE);
    }
}

fn update_carry_flag(m: &mut Machine, condition: bool) {
    if condition {
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
            assert!(!m.test_bit("SR", ZERO), "zero set");

            m.run_program(&[
                //             ;A=FF, C=0
                0x69, 0x01, // 0x0000 ADC #01
                0x00, //       0x0002 BRK
            ]);
            assert_eq!(m.reg("AC"), 0x00, "wrong AC");
            assert!(m.test_bit("SR", CARRY), "carry clear after overflow");
            assert!(m.test_bit("SR", ZERO), "zero not set");
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
            assert!(!m.test_bit("SR", ZERO), "zero set");
            m.run_program(&[
                //                   ;A=02, C=0
                0x6D, 0x04, 0x00, // 0x0000 ADC $0004
                0x00, //             0x0003 BRK
                0xFE, //             0x0004 DB #$FE
            ]);
            assert_eq!(m.reg("AC"), 0x00, "wrong AC");
            assert!(m.test_bit("SR", CARRY), "carry clear after overflow");
            assert!(m.test_bit("SR", ZERO), "zero not set");
        },
    },
    Instruction {
        mnemonic: "BEQ",
        mode: Mode::Relative,
        opcode: 0xF0,
        bytes: 2,
        cycles: 2,
        execute: |m| {
            let offset = m.fetch8();
            if m.test_bit("SR", ZERO) {
                let signed_offset = i16::from(offset.cast_signed());
                m.pc = m.pc.wrapping_add_signed(signed_offset);
            }
        },
        test: |m| {
            // Test branch forward
            m.set_bit("SR", ZERO);
            m.run_program(&[
                0xF0, 0x03, // $0000 BEQ $03
                0x00, //       $0002 BRK
                0xA9, 0x2a, // $0003 LDA #$2a
                0xA9, 0xff, // $0005 LDA #$ff
                0x00, //       $0007 BRK
            ]);
            assert_eq!(m.pc, 0x0008, "wrong PC");
            assert_eq!(m.reg("AC"), 0xff, "wrong AC");

            // Test branch backward
            m.set_bit("SR", ZERO);
            m.set_reg("AC", 0x00);
            m.run_program(&[
                0x4C, 0x06, 0x00, // $0000 JMP $0006
                0xA9, 0x2a, //       $0003 LDA #$2a
                0x00, //             $0005 BRK
                0xF0, 0xfb, //       $0006 BEQ $fb
                0x00, //             $0008 BRK
            ]);
            assert_eq!(m.pc, 0x0006, "wrong pc");
            assert_eq!(m.reg("AC"), 0x2a, "wrong AC");

            // Test branch skipped
            m.clear_bit("SR", ZERO);
            m.set_reg("AC", 0x00);
            m.run_program(&[
                0xF0, 0x02, // $0000 BEQ $02
                0xA9, 0x2a, // $0002 LDA #$2a
                0x00, //       $0004 BRK
            ]);
            assert_eq!(m.pc, 0x0005, "wrong pc");
            assert_eq!(m.reg("AC"), 0x2a, "wrong AC");
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
            if !m.test_bit("SR", ZERO) {
                let signed_offset = i16::from(offset.cast_signed());
                m.pc = m.pc.wrapping_add_signed(signed_offset);
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

            // Test branch skipped
            m.set_bit("SR", ZERO);
            m.set_bit("SR", CARRY);
            m.run_program(&[
                0xD0, 0x01, // $0000 BNE $01 (not taken)
                0x18, //       $0002 CLC
                0x00, //       $0003 BRK
            ]);
            assert_eq!(m.pc(), 0x0004, "wrong PC");
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
        mnemonic: "CLI",
        mode: Mode::Implied,
        opcode: 0x58,
        bytes: 1,
        cycles: 2,
        execute: |m| m.clear_bit("SR", INTERRUPT),
        test: |m| {
            m.set_bit("SR", INTERRUPT);
            m.run_program(&[
                0x58, // 0x0000 CLI
                0x00, // 0x0001 BRK
            ]);
            assert!(!m.test_bit("SR", INTERRUPT), "interrupt flag not cleared");
        },
    },
    Instruction {
        mnemonic: "CLV",
        mode: Mode::Implied,
        opcode: 0xB8,
        bytes: 1,
        cycles: 2,
        execute: |m| m.clear_bit("SR", OVERFLOW),
        test: |m| {
            m.set_bit("SR", OVERFLOW);
            m.run_program(&[
                0xB8, // 0x0000 CLV
                0x00, // 0x0001 BRK
            ]);
            assert!(!m.test_bit("SR", OVERFLOW), "overflow flag not cleared");
        },
    },
    Instruction {
        mnemonic: "CMP",
        mode: Mode::Immediate,
        opcode: 0xC9,
        bytes: 2,
        cycles: 2,
        execute: |m| {
            let value = m.fetch8();
            compare(m, "AC", value);
        },
        test: |m| {
            m.set_reg("AC", 0xFE);
            m.run_program(&[
                0xC9, 0xFE, // $0000 CMP #$FE
                0x00, //       $0002 BRK
            ]);
            assert!(m.test_bit("SR", CARRY), "carry flag not set");
            assert!(m.test_bit("SR", ZERO), "zero flag not cleared");
            assert!(!m.test_bit("SR", NEGATIVE), "negative flag not set");

            m.set_reg("AC", 0xFE);
            m.run_program(&[
                0xC9, 0x02, // $0000 CMP #$02
                0x00, //       $0002 BRK
            ]);
            assert!(m.test_bit("SR", CARRY), "carry flag not set");
            assert!(!m.test_bit("SR", ZERO), "zero flag not cleared");
            assert!(m.test_bit("SR", NEGATIVE), "negative flag not set");

            m.set_reg("AC", 0x05);
            m.run_program(&[
                0xC9, 0x02, // $0000 CMP #$02
                0x00, //       $0002 BRK
            ]);
            assert!(m.test_bit("SR", CARRY), "carry flag not set");
            assert!(!m.test_bit("SR", ZERO), "zero flag not set");
            assert!(!m.test_bit("SR", NEGATIVE), "negative flag not set");

            m.set_reg("AC", 0xFE);
            m.run_program(&[
                0xC9, 0xFF, // $0000 CMP #$FF
                0x00, //       $0002 BRK
            ]);
            assert!(!m.test_bit("SR", CARRY), "carry flag not cleared");
            assert!(!m.test_bit("SR", ZERO), "zero flag not set");
            assert!(m.test_bit("SR", NEGATIVE), "negative flag not set");
        },
    },
    Instruction {
        mnemonic: "CMP",
        mode: Mode::ZeroPage,
        opcode: 0xC5,
        bytes: 2,
        cycles: 3,
        execute: |m| {
            let addr = m.fetch8();
            let value = m.get8(addr.into());
            compare(m, "AC", value);
        },
        test: |m| {
            m.set_reg("AC", 0x05);
            m.set8(0x10, 0x05);
            m.run_program(&[
                0xC5, 0x10, // $0000 CMP $10
                0x00, //      $0002 BRK
            ]);
            assert!(m.test_bit("SR", CARRY), "carry flag not set");
            assert!(m.test_bit("SR", ZERO), "zero flag not set");
            assert!(!m.test_bit("SR", NEGATIVE), "negative flag not cleared");
        },
    },
    Instruction {
        mnemonic: "CMP",
        mode: Mode::Absolute,
        opcode: 0xCD,
        bytes: 3,
        cycles: 4,
        execute: |m| {
            let addr = m.fetch16();
            let value = m.get8(addr);
            compare(m, "AC", value);
        },
        test: |m| {
            m.set_reg("AC", 0x05);
            m.set8(0x1000, 0x05);
            m.run_program(&[
                0xCD, 0x00, 0x10, // $0000 CMP $1000
                0x00, //             $0003 BRK
            ]);
            assert!(m.test_bit("SR", CARRY), "carry flag not set");
            assert!(m.test_bit("SR", ZERO), "zero flag not set");
            assert!(!m.test_bit("SR", NEGATIVE), "negative flag not cleared");
        },
    },
    Instruction {
        mnemonic: "DEC",
        mode: Mode::ZeroPage,
        opcode: 0xC6,
        bytes: 2,
        cycles: 5,
        execute: |m| {
            let addr = m.fetch8();
            dec(m, addr.into());
        },
        test: |m| {
            m.set8(0x10, 0x01);
            m.run_program(&[
                0xC6, 0x10, // $0000 DEC $10
                0x00, //       $0002 BRK
            ]);
            assert_eq!(m.get8(0x10), 0x00, "wrong value");
            assert!(m.test_bit("SR", ZERO), "zero flag not set");

            m.run_program(&[
                0xC6, 0x10, // $0000 DEC $10
                0x00, //       $0002 BRK
            ]);
            assert_eq!(m.get8(0x10), 0xFF, "wrong value");
            assert!(!m.test_bit("SR", ZERO), "zero flag not cleared");
        },
    },
    Instruction {
        mnemonic: "DEC",
        mode: Mode::Absolute,
        opcode: 0xCE,
        bytes: 3,
        cycles: 6,
        execute: |m| {
            let addr = m.fetch16();
            dec(m, addr);
        },
        test: |m| {
            m.set8(0x1000, 0x01);
            m.run_program(&[
                0xCE, 0x00, 0x10, // $0000 DEC $1000
                0x00, //             $0003 BRK
            ]);
            assert_eq!(m.get8(0x1000), 0x00, "wrong value");
            assert!(m.test_bit("SR", ZERO), "zero flag not set");

            m.run_program(&[
                0xCE, 0x00, 0x10, // $0000 DEC $1000
                0x00, //             $0003 BRK
            ]);
            assert_eq!(m.get8(0x1000), 0xFF, "wrong value");
            assert!(!m.test_bit("SR", ZERO), "zero flag not cleared");
        },
    },
    Instruction {
        mnemonic: "DEX",
        mode: Mode::Implied,
        opcode: 0xCA,
        bytes: 1,
        cycles: 2,
        execute: |m| dec_reg(m, "XR"),
        test: |m| {
            m.set_reg("XR", 0x01);
            m.run_program(&[
                0xCA, // $0000 DEX
                0x00, // $0001 BRK
            ]);
            assert_eq!(m.reg("XR"), 0x00, "wrong XR");
            assert!(m.test_bit("SR", ZERO), "zero flag not set");

            m.run_program(&[
                0xCA, // $0000 DEX
                0x00, // $0002 BRK
            ]);
            assert_eq!(m.reg("XR"), 0xFF, "wrong XR");
            assert!(!m.test_bit("SR", ZERO), "zero flag not cleared");
        },
    },
    Instruction {
        mnemonic: "DEY",
        mode: Mode::Implied,
        opcode: 0x88,
        bytes: 1,
        cycles: 2,
        execute: |m| dec_reg(m, "YR"),
        test: |m| {
            m.set_reg("YR", 0x01);
            m.run_program(&[
                0x88, // $0000 DEY
                0x00, // $0001 BRK
            ]);
            assert_eq!(m.reg("YR"), 0x00, "wrong YR");
            assert!(m.test_bit("SR", ZERO), "zero flag not set");

            m.run_program(&[
                0x88, // $0000 DEY
                0x00, // $0002 BRK
            ]);
            assert_eq!(m.reg("YR"), 0xFF, "wrong YR");
            assert!(!m.test_bit("SR", ZERO), "zero flag not cleared");
        },
    },
    Instruction {
        mnemonic: "INC",
        mode: Mode::ZeroPage,
        opcode: 0xE6,
        bytes: 2,
        cycles: 5,
        execute: |m| {
            let addr = m.fetch8();
            inc(m, addr.into());
        },
        test: |m| {
            m.set8(0x10, 0xFF);
            m.run_program(&[
                0xE6, 0x10, // $0000 INC $10
                0x00, //       $0002 BRK
            ]);
            assert_eq!(m.get8(0x10), 0x00, "wrong value");
            assert!(m.test_bit("SR", ZERO), "zero flag not set");

            m.run_program(&[
                0xE6, 0x10, // $0000 INC $10
                0x00, //       $0002 BRK
            ]);
            assert_eq!(m.get8(0x10), 0x01, "wrong value");
            assert!(!m.test_bit("SR", ZERO), "zero flag not cleared");
        },
    },
    Instruction {
        mnemonic: "INC",
        mode: Mode::Absolute,
        opcode: 0xEE,
        bytes: 3,
        cycles: 6,
        execute: |m| {
            let addr = m.fetch16();
            inc(m, addr);
        },
        test: |m| {
            m.set8(0x1000, 0xFF);
            m.run_program(&[
                0xEE, 0x00, 0x10, // $0000 INC $1000
                0x00, //             $0003 BRK
            ]);
            assert_eq!(m.get8(0x1000), 0x00, "wrong value");
            assert!(m.test_bit("SR", ZERO), "zero flag not set");

            m.run_program(&[
                0xEE, 0x00, 0x10, // $0000 INC $1000
                0x00, //             $0003 BRK
            ]);
            assert_eq!(m.get8(0x1000), 0x01, "wrong value");
            assert!(!m.test_bit("SR", ZERO), "zero flag not cleared");
        },
    },
    Instruction {
        mnemonic: "INX",
        mode: Mode::Implied,
        opcode: 0xE8,
        bytes: 1,
        cycles: 2,
        execute: |m| inc_reg(m, "XR"),
        test: |m| {
            m.set_reg("XR", 0xFF);
            m.run_program(&[
                0xE8, // 0x0000 INX
                0x00, // 0x0001 BRK
            ]);
            assert_eq!(m.reg("XR"), 0x00, "wrong XR");
            assert!(m.test_bit("SR", ZERO), "zero flag not set");

            m.run_program(&[
                0xE8, // 0x0000 INX
                0x00, // 0x0001 BRK
            ]);
            assert_eq!(m.reg("XR"), 0x01, "wrong XR");
            assert!(!m.test_bit("SR", ZERO), "zero flag not cleared");
        },
    },
    Instruction {
        mnemonic: "INY",
        mode: Mode::Implied,
        opcode: 0xC8,
        bytes: 1,
        cycles: 2,
        execute: |m| inc_reg(m, "YR"),
        test: |m| {
            m.set_reg("YR", 0xFF);
            m.run_program(&[
                0xC8, // 0x0000 INY
                0x00, // 0x0001 BRK
            ]);
            assert_eq!(m.reg("YR"), 0x00, "wrong YR");
            assert!(m.test_bit("SR", ZERO), "zero flag not set");

            m.run_program(&[
                0xC8, // 0x0000 INY
                0x00, // 0x0001 BRK
            ]);
            assert_eq!(m.reg("YR"), 0x01, "wrong YR");
            assert!(!m.test_bit("SR", ZERO), "zero flag not cleared");
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
            let value = m.fetch8();
            load_reg(m, "AC", value);
        },
        test: |m| {
            m.run_program(&[
                0xA9, 0x00, // 0x0000 LDA #$00
                0x00, //       0x0002 BRK
            ]);
            assert_eq!(m.reg("AC"), 0x00, "wrong AC");
            assert!(m.test_bit("SR", ZERO), "zero flag not set");
            assert!(!m.test_bit("SR", NEGATIVE), "negative flag not cleared");

            m.run_program(&[
                0xA9, 0xFF, // 0x0000 LDA #$FF
                0x00, //       0x0002 BRK
            ]);
            assert_eq!(m.reg("AC"), 0xFF, "wrong AC");
            assert!(!m.test_bit("SR", ZERO), "zero flag not cleared");
            assert!(m.test_bit("SR", NEGATIVE), "negative flag not set");
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
            let value = m.get8(addr.into());
            load_reg(m, "AC", value);
        },
        test: |m| {
            m.run_program(&[
                0xA5, 0x03, // 0x0000 LDA $03
                0x00, //       0x0002 BRK
                0x00, //       0x0003 DB #$00
            ]);
            assert_eq!(m.reg("AC"), 0x00, "wrong AC");
            assert!(m.test_bit("SR", ZERO), "zero flag not set");
            assert!(!m.test_bit("SR", NEGATIVE), "negative flag not cleared");

            m.run_program(&[
                0xA5, 0x03, // 0x0000 LDA $03
                0x00, //       0x0002 BRK
                0xFF, //       0x0003 DB #$FF
            ]);
            assert_eq!(m.reg("AC"), 0xFF, "wrong AC");
            assert!(!m.test_bit("SR", ZERO), "zero flag not cleared");
            assert!(m.test_bit("SR", NEGATIVE), "negative flag not set");
        },
    },
    Instruction {
        mnemonic: "LDA",
        mode: Mode::ZeroPageX,
        opcode: 0xB5,
        bytes: 2,
        cycles: 4,
        execute: |m| {
            let base = m.fetch8();
            let addr = base.wrapping_add(m.reg("XR"));
            let value = m.get8(addr.into());
            load_reg(m, "AC", value);
        },
        test: |m| {
            m.run_program(&[
                0xA2, 0x02, // $0000 LDX #$02
                0xB5, 0x03, // $0002 LDA $03,X
                0x00, //       $0004 BRK
                0x00, //       $0005 DB #$00
            ]);
            assert_eq!(m.reg("AC"), 0x00, "wrong AC");
            assert!(m.test_bit("SR", ZERO), "zero flag not set");
            assert!(!m.test_bit("SR", NEGATIVE), "negative flag not cleared");

            m.run_program(&[
                0xA2, 0x00, // $0000 LDX #$00
                0xB5, 0x05, // $0002 LDA $05,X
                0x00, //       $0004 BRK
                0xFF, //       $0005 DB #$FF
            ]);
            assert_eq!(m.reg("AC"), 0xFF, "wrong AC");
            assert!(!m.test_bit("SR", ZERO), "zero flag not cleared");
            assert!(m.test_bit("SR", NEGATIVE), "negative flag not cleared");

            m.set8(0x08, 0x42);
            m.run_program(&[
                0xA2, 0x03, // $0000 LDX #$03
                0xB5, 0x05, // $0002 LDA $05,X
                0x00, //       $0004 BRK
            ]);
            assert_eq!(m.reg("AC"), 0x42, "wrong AC");
            assert!(!m.test_bit("SR", ZERO), "zero flag not cleared");
            assert!(!m.test_bit("SR", NEGATIVE), "negative flag not cleared");

            m.set8(0x0F, 0x2a);
            m.run_program(&[
                0xA2, 0xFF, // $0000 LDX #$FF
                0xB5, 0x10, // $0002 LDA $10,X ; $10 + $FF = $0F
                0x00, //       $0004 BRK
            ]);
            assert_eq!(m.reg("AC"), 0x2a, "wrong AC");
            assert!(!m.test_bit("SR", ZERO), "zero flag not cleared");
            assert!(!m.test_bit("SR", NEGATIVE), "negative flag not set");
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
            let value = m.get8(addr);
            load_reg(m, "AC", value);
        },
        test: |m| {
            m.run_program(&[
                0xAD, 0x04, 0x00, // 0x0000 LDA $0004
                0x00, //             0x0003 BRK
                0x00, //             0x0004 DB #$00
            ]);
            assert_eq!(m.reg("AC"), 0x00, "wrong AC");
            assert!(m.test_bit("SR", ZERO), "zero flag not set");
            assert!(!m.test_bit("SR", NEGATIVE), "negative flag not cleared");

            m.run_program(&[
                0xAD, 0x04, 0x00, // 0x0000 LDA $0004
                0x00, //             0x0003 BRK
                0xFF, //             0x0004 DB #$FF
            ]);
            assert_eq!(m.reg("AC"), 0xFF, "wrong AC");
            assert!(!m.test_bit("SR", ZERO), "zero flag not cleared");
            assert!(m.test_bit("SR", NEGATIVE), "negative flag not set");
        },
    },
    Instruction {
        mnemonic: "LDX",
        mode: Mode::Immediate,
        opcode: 0xA2,
        bytes: 2,
        cycles: 2,
        execute: |m| {
            let value = m.fetch8();
            load_reg(m, "XR", value);
        },
        test: |m| {
            m.run_program(&[
                0xA2, 0x00, // 0x0000 LDX #$00
                0x00, //       0x0002 BRK
            ]);
            assert_eq!(m.reg("XR"), 0x00, "wrong XR");
            assert!(m.test_bit("SR", ZERO), "zero flag not set");
            assert!(!m.test_bit("SR", NEGATIVE), "negative flag not cleared");

            m.run_program(&[
                0xA2, 0xFF, // 0x0000 LDX #$FF
                0x00, //       0x0002 BRK
            ]);
            assert_eq!(m.reg("XR"), 0xFF, "wrong XR");
            assert!(!m.test_bit("SR", ZERO), "zero flag not cleared");
            assert!(m.test_bit("SR", NEGATIVE), "negative flag not set");
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
            let value = m.get8(addr.into());
            load_reg(m, "XR", value);
        },
        test: |m| {
            m.run_program(&[
                0xA6, 0x03, // 0x0000 LDX $03
                0x00, //       0x0002 BRK
                0x00, //       0x0003 DB #$00
            ]);
            assert_eq!(m.reg("XR"), 0x00, "wrong XR");
            assert!(m.test_bit("SR", ZERO), "zero flag not set");
            assert!(!m.test_bit("SR", NEGATIVE), "negative flag not cleared");

            m.run_program(&[
                0xA6, 0x03, // 0x0000 LDX $03
                0x00, //       0x0002 BRK
                0xFF, //       0x0003 DB #$FF
            ]);
            assert_eq!(m.reg("XR"), 0xFF, "wrong XR");
            assert!(!m.test_bit("SR", ZERO), "zero flag not cleared");
            assert!(m.test_bit("SR", NEGATIVE), "negative flag not set");
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
            let value = m.get8(addr);
            load_reg(m, "XR", value);
        },
        test: |m| {
            m.run_program(&[
                0xAE, 0x04, 0x00, // 0x0000 LDX $0004
                0x00, //             0x0003 BRK
                0x00, //             0x0004 DB #$00
            ]);
            assert_eq!(m.reg("XR"), 0x00, "wrong XR");
            assert!(m.test_bit("SR", ZERO), "zero flag not set");
            assert!(!m.test_bit("SR", NEGATIVE), "negative flag not cleared");

            m.run_program(&[
                0xAE, 0x04, 0x00, // 0x0000 LDX $0004
                0x00, //             0x0003 BRK
                0xFF, //             0x0004 DB #$FF
            ]);
            assert_eq!(m.reg("XR"), 0xFF, "wrong XR");
            assert!(!m.test_bit("SR", ZERO), "zero flag not cleared");
            assert!(m.test_bit("SR", NEGATIVE), "negative flag not set");
        },
    },
    Instruction {
        mnemonic: "LDY",
        mode: Mode::Immediate,
        opcode: 0xA0,
        bytes: 2,
        cycles: 2,
        execute: |m| {
            let value = m.fetch8();
            load_reg(m, "YR", value);
        },
        test: |m| {
            m.run_program(&[
                0xA0, 0x00, // 0x0000 LDY #$00
                0x00, //       0x0002 BRK
            ]);
            assert_eq!(m.reg("YR"), 0x00, "wrong YR");
            assert!(m.test_bit("SR", ZERO), "zero flag not set");
            assert!(!m.test_bit("SR", NEGATIVE), "negative flag not cleared");

            m.run_program(&[
                0xA0, 0xFF, // 0x0000 LDY #$FF
                0x00, //       0x0002 BRK
            ]);
            assert_eq!(m.reg("YR"), 0xFF, "wrong YR");
            assert!(!m.test_bit("SR", ZERO), "zero flag not cleared");
            assert!(m.test_bit("SR", NEGATIVE), "negative flag not set");
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
            let value = m.get8(addr.into());
            load_reg(m, "YR", value);
        },
        test: |m| {
            m.run_program(&[
                0xA4, 0x03, // 0x0000 LDY $03
                0x00, //       0x0002 BRK
                0x00, //       0x0003 DB #$00
            ]);
            assert_eq!(m.reg("YR"), 0x00, "wrong YR");
            assert!(m.test_bit("SR", ZERO), "zero flag not set");
            assert!(!m.test_bit("SR", NEGATIVE), "negative flag not cleared");

            m.run_program(&[
                0xA4, 0x03, // 0x0000 LDY $03
                0x00, //       0x0002 BRK
                0xFF, //       0x0003 DB #$FF
            ]);
            assert_eq!(m.reg("YR"), 0xFF, "wrong YR");
            assert!(!m.test_bit("SR", ZERO), "zero flag not cleared");
            assert!(m.test_bit("SR", NEGATIVE), "negative flag not set");
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
            let value = m.get8(addr);
            load_reg(m, "YR", value);
        },
        test: |m| {
            m.run_program(&[
                0xAC, 0x04, 0x00, // 0x0000 LDY $0004
                0x00, //             0x0003 BRK
                0x00, //             0x0004 DB #$00
            ]);
            assert_eq!(m.reg("YR"), 0x00, "wrong YR");
            assert!(m.test_bit("SR", ZERO), "zero flag not set");
            assert!(!m.test_bit("SR", NEGATIVE), "negative flag not cleared");

            m.run_program(&[
                0xAC, 0x04, 0x00, // 0x0000 LDY $0004
                0x00, //             0x0003 BRK
                0xFF, //             0x0004 DB #$FF
            ]);
            assert_eq!(m.reg("YR"), 0xFF, "wrong YR");
            assert!(!m.test_bit("SR", ZERO), "zero flag not cleared");
            assert!(m.test_bit("SR", NEGATIVE), "negative flag not set");
        },
    },
    Instruction {
        mnemonic: "NOP",
        mode: Mode::Implied,
        opcode: 0xEA,
        bytes: 1,
        cycles: 2,
        execute: |_m| {},
        test: |m| {
            m.run_program(&[
                0xEA, // 0x0000 NOP
                0x00, // 0x0001 BRK
            ]);
            assert_eq!(m.pc, 0x0002, "wrong PC");
        },
    },
    Instruction {
        mnemonic: "SEC",
        mode: Mode::Implied,
        opcode: 0x38,
        bytes: 1,
        cycles: 2,
        execute: |m| m.set_bit("SR", CARRY),
        test: |m| {
            m.run_program(&[
                0x38, // 0x0000 SEC
                0x00, // 0x0001 BRK
            ]);
            assert!(m.test_bit("SR", CARRY), "carry flag not set");
        },
    },
    Instruction {
        mnemonic: "SED",
        mode: Mode::Implied,
        opcode: 0xF8,
        bytes: 1,
        cycles: 2,
        execute: |m| m.set_bit("SR", DECIMAL),
        test: |m| {
            m.run_program(&[
                0xF8, // 0x0000 SED
                0x00, // 0x0001 BRK
            ]);
            assert!(m.test_bit("SR", DECIMAL), "decimal flag not set");
        },
    },
    Instruction {
        mnemonic: "SEI",
        mode: Mode::Implied,
        opcode: 0x78,
        bytes: 1,
        cycles: 2,
        execute: |m| m.set_bit("SR", INTERRUPT),
        test: |m| {
            m.run_program(&[
                0x78, // 0x0000 SEI
                0x00, // 0x0001 BRK
            ]);
            assert!(m.test_bit("SR", INTERRUPT), "interrupt  flag not set");
        },
    },
    Instruction {
        mnemonic: "STA",
        mode: Mode::ZeroPage,
        opcode: 0x85,
        bytes: 2,
        cycles: 3,
        execute: |m| {
            let addr = m.fetch8();
            let value = m.reg("AC");
            m.set8(addr.into(), value);
        },
        test: |m| {
            m.run_program(&[
                0xA9, 0x42, // $0000 LDA #$42
                0x85, 0x05, // $0002 STA $05
                0x00, //       $0004 BRK
                0x00, //       $0005 DB #$00
            ]);
            assert_eq!(m.get8(0x05), 0x42, "wrong value at address");
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
                0xA9, 0x42, //       0x0000 LDA #$42
                0x8D, 0x06, 0x00, // 0x0002 STA $0006
                0x00, //             0x0005 BRK
                0x00, //             0x0006 DB #$00
            ]);
            assert_eq!(m.get8(0x0006), 0x42, "wrong value at address");
        },
    },
    Instruction {
        mnemonic: "STX",
        mode: Mode::ZeroPage,
        opcode: 0x86,
        bytes: 2,
        cycles: 3,
        execute: |m| {
            let addr = m.fetch8();
            let op = m.reg("XR");
            m.set8(addr.into(), op);
        },
        test: |m| {
            m.run_program(&[
                0xA2, 0x42, // $0000 LDX #$42
                0x86, 0x05, // $0002 STX $05
                0x00, //       $0004 BRK
                0x00, //       $0005 DB #$00
            ]);
            assert_eq!(m.get8(0x05), 0x42, "wrong value at address");
        },
    },
    Instruction {
        mnemonic: "STX",
        mode: Mode::Absolute,
        opcode: 0x8E,
        bytes: 3,
        cycles: 4,
        execute: |m| {
            let addr = m.fetch16();
            let op = m.reg("XR");
            m.set8(addr, op);
        },
        test: |m| {
            m.run_program(&[
                0xA2, 0x42, //       $0000 LDX #$42
                0x8E, 0x06, 0x00, // $0002 STX $0006
                0x00, //             $0005 BRK
                0x00, //             $0006 DB #$00
            ]);
            assert_eq!(m.get8(0x0006), 0x42, "wrong value at address");
        },
    },
    Instruction {
        mnemonic: "STY",
        mode: Mode::ZeroPage,
        opcode: 0x84,
        bytes: 2,
        cycles: 3,
        execute: |m| {
            let addr = m.fetch8();
            let op = m.reg("YR");
            m.set8(addr.into(), op);
        },
        test: |m| {
            m.run_program(&[
                0xA0, 0x42, // $0000 LDY #$42
                0x84, 0x05, // $0002 STY $05
                0x00, //       $0004 BRK
                0x00, //       $0005 DB #$00
            ]);
            assert_eq!(m.get8(0x05), 0x42, "wrong value at address");
        },
    },
    Instruction {
        mnemonic: "STY",
        mode: Mode::Absolute,
        opcode: 0x8C,
        bytes: 3,
        cycles: 4,
        execute: |m| {
            let addr = m.fetch16();
            let op = m.reg("YR");
            m.set8(addr, op);
        },
        test: |m| {
            m.run_program(&[
                0xA0, 0x42, //       $0000 LDY #$42
                0x8C, 0x06, 0x00, // $0002 STY $0006
                0x00, //             $0005 BRK
                0x00, //             $0006 DB #$00
            ]);
            assert_eq!(m.get8(0x0006), 0x42, "wrong value at address");
        },
    },
    Instruction {
        mnemonic: "TAX",
        mode: Mode::Implied,
        opcode: 0xAA,
        bytes: 1,
        cycles: 2,
        execute: |m| load_reg(m, "XR", m.reg("AC")),
        test: |m| {
            m.set_reg("AC", 0x42);
            m.set_bit("SR", ZERO);
            m.run_program(&[
                0xAA, // $0000 TAX
                0x00, // $0001 BRK
            ]);
            assert_eq!(m.reg("XR"), 0x42, "wrong XR");
            assert!(!m.test_bit("SR", ZERO), "zero flag set");
            assert!(!m.test_bit("SR", NEGATIVE), "negative flag not cleared");

            m.set_reg("AC", 0x80);
            m.set_bit("SR", ZERO);
            m.run_program(&[
                0xAA, // $0000 TAX
                0x00, // $0001 BRK
            ]);
            assert_eq!(m.reg("XR"), 0x80, "wrong XR");
            assert!(!m.test_bit("SR", ZERO), "zero flag set");
            assert!(m.test_bit("SR", NEGATIVE), "negative flag not set");

            m.set_reg("AC", 0x00);
            m.run_program(&[
                0xAA, // $0000 TAX
                0x00, // $0001 BRK
            ]);
            assert_eq!(m.reg("XR"), 0x00, "wrong XR");
            assert!(m.test_bit("SR", ZERO), "zero flag not set");
            assert!(!m.test_bit("SR", NEGATIVE), "negative flag not cleared");
        },
    },
    Instruction {
        mnemonic: "TAY",
        mode: Mode::Implied,
        opcode: 0xA8,
        bytes: 1,
        cycles: 2,
        execute: |m| load_reg(m, "YR", m.reg("AC")),
        test: |m| {
            m.set_reg("AC", 0x42);
            m.set_bit("SR", ZERO);
            m.run_program(&[
                0xA8, // $0000 TAY
                0x00, // $0001 BRK
            ]);
            assert_eq!(m.reg("YR"), 0x42, "wrong YR");
            assert!(!m.test_bit("SR", ZERO), "zero flag set");
            assert!(!m.test_bit("SR", NEGATIVE), "negative flag not cleared");

            m.set_reg("AC", 0x80);
            m.set_bit("SR", ZERO);
            m.run_program(&[
                0xA8, // $0000 TAY
                0x00, // $0001 BRK
            ]);
            assert_eq!(m.reg("YR"), 0x80, "wrong YR");
            assert!(!m.test_bit("SR", ZERO), "zero flag set");
            assert!(m.test_bit("SR", NEGATIVE), "negative flag not set");

            m.set_reg("AC", 0x00);
            m.run_program(&[
                0xA8, // $0000 TAY
                0x00, // $0001 BRK
            ]);
            assert_eq!(m.reg("YR"), 0x00, "wrong YR");
            assert!(m.test_bit("SR", ZERO), "zero flag not set");
            assert!(!m.test_bit("SR", NEGATIVE), "negative flag not cleared");
        },
    },
    Instruction {
        mnemonic: "TSX",
        mode: Mode::Implied,
        opcode: 0xBA,
        bytes: 1,
        cycles: 2,
        execute: |m| load_reg(m, "XR", m.reg("SP")),
        test: |m| {
            m.set_reg("SP", 0x42);
            m.set_bit("SR", ZERO);
            m.run_program(&[
                0xBA, // $0000 TSX
                0x00, // $0001 BRK
            ]);
            assert_eq!(m.reg("XR"), 0x42, "wrong XR");
            assert!(!m.test_bit("SR", ZERO), "zero flag set");
            assert!(!m.test_bit("SR", NEGATIVE), "negative flag not cleared");

            m.set_reg("SP", 0x80);
            m.set_bit("SR", ZERO);
            m.run_program(&[
                0xBA, // $0000 TSX
                0x00, // $0001 BRK
            ]);
            assert_eq!(m.reg("XR"), 0x80, "wrong XR");
            assert!(!m.test_bit("SR", ZERO), "zero flag set");
            assert!(m.test_bit("SR", NEGATIVE), "negative flag not set");

            m.set_reg("SP", 0x00);
            m.run_program(&[
                0xBA, // $0000 TSX
                0x00, // $0001 BRK
            ]);
            assert_eq!(m.reg("XR"), 0x00, "wrong XR");
            assert!(m.test_bit("SR", ZERO), "zero flag not set");
            assert!(!m.test_bit("SR", NEGATIVE), "negative flag not cleared");
        },
    },
    Instruction {
        mnemonic: "TXA",
        mode: Mode::Implied,
        opcode: 0x8A,
        bytes: 1,
        cycles: 2,
        execute: |m| load_reg(m, "AC", m.reg("XR")),
        test: |m| {
            m.set_reg("XR", 0x42);
            m.set_bit("SR", ZERO);
            m.run_program(&[
                0x8A, // $0000 TXA
                0x00, // $0001 BRK
            ]);
            assert_eq!(m.reg("AC"), 0x42, "wrong AC");
            assert!(!m.test_bit("SR", ZERO), "zero flag set");
            assert!(!m.test_bit("SR", NEGATIVE), "negative flag not cleared");

            m.set_reg("XR", 0x80);
            m.set_bit("SR", ZERO);
            m.run_program(&[
                0x8A, // $0000 TXA
                0x00, // $0001 BRK
            ]);
            assert_eq!(m.reg("AC"), 0x80, "wrong AC");
            assert!(!m.test_bit("SR", ZERO), "zero flag set");
            assert!(m.test_bit("SR", NEGATIVE), "negative flag not set");

            m.set_reg("XR", 0x00);
            m.run_program(&[
                0x8A, // $0000 TXA
                0x00, // $0001 BRK
            ]);
            assert_eq!(m.reg("AC"), 0x00, "wrong AC");
            assert!(m.test_bit("SR", ZERO), "zero flag not set");
            assert!(!m.test_bit("SR", NEGATIVE), "negative flag not cleared");
        },
    },
    Instruction {
        mnemonic: "TXS",
        mode: Mode::Implied,
        opcode: 0x9A,
        bytes: 1,
        cycles: 2,
        execute: |m| m.set_reg("SP", m.reg("XR")),
        test: |m| {
            m.set_reg("XR", 0x42);
            m.run_program(&[
                0x9A, // $0000 TXS
                0x00, // $0001 BRK
            ]);
            assert_eq!(m.reg("SP"), 0x42, "wrong SP");
        },
    },
    Instruction {
        mnemonic: "TYA",
        mode: Mode::Implied,
        opcode: 0x98,
        bytes: 1,
        cycles: 2,
        execute: |m| load_reg(m, "AC", m.reg("YR")),
        test: |m| {
            m.set_reg("YR", 0x42);
            m.set_bit("SR", ZERO);
            m.run_program(&[
                0x98, // $0000 TYA
                0x00, // $0001 BRK
            ]);
            assert_eq!(m.reg("AC"), 0x42, "wrong AC");
            assert!(!m.test_bit("SR", ZERO), "zero flag set");
            assert!(!m.test_bit("SR", NEGATIVE), "negative flag not cleared");

            m.set_reg("YR", 0x80);
            m.set_bit("SR", ZERO);
            m.run_program(&[
                0x98, // $0000 TYA
                0x00, // $0001 BRK
            ]);
            assert_eq!(m.reg("AC"), 0x80, "wrong AC");
            assert!(!m.test_bit("SR", ZERO), "zero flag set");
            assert!(m.test_bit("SR", NEGATIVE), "negative flag not set");

            m.set_reg("YR", 0x00);
            m.run_program(&[
                0x98, // $0000 TYA
                0x00, // $0001 BRK
            ]);
            assert_eq!(m.reg("AC"), 0x00, "wrong AC");
            assert!(m.test_bit("SR", ZERO), "zero flag not set");
            assert!(!m.test_bit("SR", NEGATIVE), "negative flag not cleared");
        },
    },
];
