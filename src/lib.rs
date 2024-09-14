#![allow(unused)]

use std::num::TryFromIntError;

type Word = u32;

#[derive(Debug)]
enum Error {
    OpcodeUnknown(u32),
    ImmediateValueInvalid(TryFromIntError),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Default, PartialEq)]
struct Machine {
    pc: Word,
    sp: Word,
    a: [Word; 16],
    ra: Word,
    x0: Word,
}

impl Machine {
    fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, PartialEq)]
enum Opcode {
    LoadImmediate,
    Add,
}

impl TryFrom<Word> for Opcode {
    type Error = Error;

    fn try_from(value: Word) -> Result<Self> {
        match value {
            0b00001 => Ok(Opcode::LoadImmediate),
            0b00010 => Ok(Opcode::Add),
            _ => Err(Error::OpcodeUnknown(value)),
        }
    }
}

#[derive(Debug, PartialEq)]
struct Instruction {
    opcode: Opcode,
    rd: u32,
    rs1: u32,
    rs2: u32,
    imm: u16,
}

impl Instruction {
    fn decode(word: Word) -> Result<Self> {
        let opcode = Opcode::try_from(word & 0x1f)?;
        let rd = (word >> 5) & 0xf;
        let rs1 = (word >> 9) & 0xf;
        let rs2 = (word >> 13) & 0xf;
        let imm = (word >> 17)
            .try_into()
            .map_err(Error::ImmediateValueInvalid)?;
        Ok(Self {
            opcode,
            rd,
            rs1,
            rs2,
            imm,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::assert_ok_eq;

    #[test]
    fn new_returns_initialized_machine() {
        let want = Machine {
            sp: 0u32,
            pc: 0u32,
            a: [0u32; 16],
            ra: 0u32,
            x0: 0u32,
        };
        let got = Machine::new();
        assert_eq!(want, got);
    }

    #[test]
    fn decode_parses_an_opcode_from_its_binary_representation() {
        struct TestCase {
            word: Word,
            want: Opcode,
        }
        let cases = [
            TestCase {
                word: 0b00001,
                want: Opcode::LoadImmediate,
            },
            TestCase {
                word: 0b00010,
                want: Opcode::Add,
            },
        ];
        for case in cases {
            assert_ok_eq!(Opcode::try_from(case.word), case.want);
        }
    }

    #[test]
    fn decode_parses_an_instruction_from_a_32_bit_word() {
        struct TestCase {
            word: u32,
            want: Instruction,
        }
        let cases = [TestCase {
            word: 0b0000_0000_0000_0000_0100_0010_0000_0010,
            want: Instruction {
                opcode: Opcode::Add,
                rd: 0,
                rs1: 1,
                rs2: 2,
                imm: 0,
            },
        }];
        for case in cases {
            assert_ok_eq!(Instruction::decode(case.word), case.want);
        }
    }
}
