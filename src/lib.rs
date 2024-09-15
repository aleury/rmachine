#![allow(unused, clippy::cast_lossless)]

use std::{collections::HashMap, num::TryFromIntError};

#[derive(Debug, PartialEq)]
enum Error {
    OpcodeUnknown(u32),
    RegisterUnknown(u32),
    ImmediateValue(TryFromIntError),
}

type Result<T> = std::result::Result<T, Error>;

type Word = u32;

type Memory = HashMap<Word, Word>;

type Registers = HashMap<RegisterID, Word>;

#[derive(Debug, Default, Eq, PartialEq)]
struct Machine {
    pc: Word,
    mem: Memory,
    regs: Registers,
}

impl Machine {
    fn new() -> Self {
        Self::default()
    }

    fn load_program(&mut self, program: &[Word]) {
        #[allow(clippy::cast_possible_truncation)]
        for (address, &word) in program.iter().enumerate() {
            self.mem.insert(address as u32, word);
        }
    }

    fn run(&mut self) -> Result<()> {
        let word = *self.mem.get(&self.pc).unwrap_or(&Word::default());
        let instruction = Instruction::try_from(word)?;
        self.pc += 1;

        match instruction.opcode {
            Opcode::LoadImmediate => {
                self.regs.insert(instruction.rd, instruction.imm as u32);
            }
            Opcode::Add => {
                let rs1 = *self.regs.get(&instruction.rs1).unwrap_or(&Word::default());
                let rs2 = *self.regs.get(&instruction.rs2).unwrap_or(&Word::default());
                let imm = instruction.imm as Word;
                self.regs.insert(instruction.rd, rs1 + rs2 + imm);
            }
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq)]
enum Opcode {
    LoadImmediate,
    Add,
}

impl TryFrom<Word> for Opcode {
    type Error = Error;

    fn try_from(word: Word) -> Result<Self> {
        match word {
            0b00001 => Ok(Opcode::LoadImmediate),
            0b00010 => Ok(Opcode::Add),
            _ => Err(Error::OpcodeUnknown(word)),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash, PartialOrd)]
enum RegisterID {
    X0,
    A0,
    A1,
    A2,
    A3,
    A4,
    A5,
    A6,
    A7,
    A8,
    A9,
    A10,
    A11,
    A12,
    RA,
    SP,
}

impl TryFrom<Word> for RegisterID {
    type Error = Error;

    fn try_from(word: Word) -> Result<Self> {
        match word {
            0b0000 => Ok(RegisterID::X0),
            0b0001 => Ok(RegisterID::A0),
            0b0010 => Ok(RegisterID::A1),
            0b0011 => Ok(RegisterID::A2),
            0b0100 => Ok(RegisterID::A3),
            0b0101 => Ok(RegisterID::A4),
            0b0110 => Ok(RegisterID::A5),
            0b0111 => Ok(RegisterID::A6),
            0b1000 => Ok(RegisterID::A7),
            0b1001 => Ok(RegisterID::A8),
            0b1010 => Ok(RegisterID::A9),
            0b1011 => Ok(RegisterID::A10),
            0b1100 => Ok(RegisterID::A11),
            0b1101 => Ok(RegisterID::A12),
            0b1110 => Ok(RegisterID::RA),
            0b1111 => Ok(RegisterID::SP),
            _ => Err(Error::RegisterUnknown(word)),
        }
    }
}

#[derive(Debug, PartialEq)]
struct Instruction {
    opcode: Opcode,
    rd: RegisterID,
    rs1: RegisterID,
    rs2: RegisterID,
    imm: u16,
}

impl TryFrom<Word> for Instruction {
    type Error = Error;

    fn try_from(word: Word) -> Result<Self> {
        let opcode = (word & 0x1f).try_into()?;
        let rd = ((word >> 5) & 0xf).try_into()?;
        let rs1 = ((word >> 9) & 0xf).try_into()?;
        let rs2 = ((word >> 13) & 0xf).try_into()?;
        let imm = (word >> 17).try_into().map_err(Error::ImmediateValue)?;
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
    use claims::{assert_err_eq, assert_ok_eq, assert_some_eq};

    #[test]
    fn new_returns_initialized_machine() {
        let want = Machine {
            pc: 0u32,
            mem: Memory::default(),
            regs: Registers::default(),
        };
        let got = Machine::new();
        assert_eq!(want, got);
    }

    #[test]
    fn parsing_an_invalid_opcode_returns_an_error() {
        assert_err_eq!(Opcode::try_from(0), Error::OpcodeUnknown(0));
    }

    #[test]
    fn opcodes_can_be_decoded_from_binary_representation() {
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
    fn parsing_an_invalid_register_returns_an_error() {
        assert_err_eq!(
            RegisterID::try_from(0b10000),
            Error::RegisterUnknown(0b10000)
        );
    }

    #[test]
    fn registers_can_be_decoded_from_binary_representation() {
        struct TestCase {
            word: Word,
            want: RegisterID,
        }
        let cases = [
            TestCase {
                word: 0b0000,
                want: RegisterID::X0,
            },
            TestCase {
                word: 0b0001,
                want: RegisterID::A0,
            },
            TestCase {
                word: 0b0010,
                want: RegisterID::A1,
            },
            TestCase {
                word: 0b0011,
                want: RegisterID::A2,
            },
            TestCase {
                word: 0b0100,
                want: RegisterID::A3,
            },
            TestCase {
                word: 0b0101,
                want: RegisterID::A4,
            },
            TestCase {
                word: 0b0110,
                want: RegisterID::A5,
            },
            TestCase {
                word: 0b0111,
                want: RegisterID::A6,
            },
            TestCase {
                word: 0b1000,
                want: RegisterID::A7,
            },
            TestCase {
                word: 0b1001,
                want: RegisterID::A8,
            },
            TestCase {
                word: 0b1010,
                want: RegisterID::A9,
            },
            TestCase {
                word: 0b1011,
                want: RegisterID::A10,
            },
            TestCase {
                word: 0b1100,
                want: RegisterID::A11,
            },
            TestCase {
                word: 0b1101,
                want: RegisterID::A12,
            },
            TestCase {
                word: 0b1110,
                want: RegisterID::RA,
            },
            TestCase {
                word: 0b1111,
                want: RegisterID::SP,
            },
        ];
        for case in cases {
            assert_ok_eq!(RegisterID::try_from(case.word), case.want);
        }
    }

    #[test]
    fn instructions_can_be_decoded_from_a_32_bit_words() {
        struct TestCase {
            word: Word,
            want: Instruction,
        }
        let cases = [
            TestCase {
                word: 0b0000_0000_0000_0100_0000_0000_0010_0001,
                want: Instruction {
                    opcode: Opcode::LoadImmediate,
                    rd: RegisterID::A0,
                    rs1: RegisterID::X0,
                    rs2: RegisterID::X0,
                    imm: 2,
                },
            },
            TestCase {
                word: 0b0000_0000_0000_0000_0110_0100_0010_0010,
                want: Instruction {
                    opcode: Opcode::Add,
                    rd: RegisterID::A0,
                    rs1: RegisterID::A1,
                    rs2: RegisterID::A2,
                    imm: 0,
                },
            },
        ];
        for case in cases {
            assert_ok_eq!(Instruction::try_from(case.word), case.want);
        }
    }

    #[test]
    fn run_executes_a_load_immediate_instruction() {
        let mut machine = Machine::new();
        machine.load_program(&[0b0000_0000_0000_0100_0000_0000_0010_0001]);

        machine.run();

        assert_eq!(machine.pc, 1);
        assert_some_eq!(machine.regs.get(&RegisterID::A0), &2);
    }

    #[test]
    fn run_executes_an_add_instruction() {
        let mut machine = Machine::new();
        machine.regs.insert(RegisterID::A1, 2);
        machine.regs.insert(RegisterID::A2, 3);
        machine.load_program(&[0b0000_0000_0000_0010_0110_0100_0010_0010]);

        machine.run();

        assert_eq!(machine.pc, 1);
        assert_some_eq!(machine.regs.get(&RegisterID::A0), &6);
    }
}
