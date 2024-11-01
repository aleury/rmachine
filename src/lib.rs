#![allow(unused, clippy::cast_lossless, clippy::cast_possible_truncation)]
use std::{collections::HashMap, io::Write, num::TryFromIntError};

#[derive(Debug, PartialEq)]
enum Error {
    OpcodeUnknown(u32),
    RegisterUnknown(u32),
    SyscallUnknown(u32),
    ImmediateValue(TryFromIntError),
}

type Result<T> = std::result::Result<T, Error>;

type Word = u32;

type Address = u32;

#[derive(Debug, Default, Eq, PartialEq)]
struct Memory {
    inner: HashMap<Address, Word>,
}

impl Memory {
    fn get(&self, addr: Address) -> Word {
        *self.inner.get(&addr).unwrap_or(&Word::default())
    }

    fn read(&self, addr: Address, len: usize) -> Vec<Word> {
        let mut data = Vec::new();
        for offset in 0..len {
            data.push(self.get(addr + offset as Word));
        }
        data
    }
}

impl<const N: usize> From<[(Address, Word); N]> for Memory {
    fn from(values: [(Address, Word); N]) -> Self {
        Self {
            inner: HashMap::from(values),
        }
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
struct Registers {
    inner: HashMap<RegisterID, Word>,
}

impl Registers {
    fn get(&self, reg: &RegisterID) -> Word {
        *self.inner.get(reg).unwrap_or(&Word::default())
    }

    fn set(&mut self, reg: RegisterID, value: Word) {
        let value = match reg {
            RegisterID::X0 => 0,
            _ => value,
        };
        self.inner.insert(reg, value);
    }
}

impl<const N: usize> From<[(RegisterID, Word); N]> for Registers {
    fn from(values: [(RegisterID, Word); N]) -> Self {
        Self {
            inner: HashMap::from(values),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Machine {
    pc: Word,
    mem: Memory,
    regs: Registers,
    out: Vec<u8>,
}

impl Default for Machine {
    fn default() -> Self {
        Self {
            pc: 0,
            out: vec![],
            mem: Memory::default(),
            regs: Registers::default(),
        }
    }
}

impl Machine {
    fn new() -> Self {
        Self::default()
    }

    fn next(&mut self) -> Result<Instruction> {
        let word = self.mem.get(self.pc);
        Instruction::try_from(word)
    }

    fn write_byte(&mut self, data: u8) {
        self.out.push(data);
    }

    fn run(&mut self) -> Result<()> {
        loop {
            let instruction = self.next()?;
            self.pc += 1;

            match instruction.opcode {
                Opcode::LoadImmediate => {
                    self.regs.set(instruction.rd, instruction.imm as Word);
                }
                Opcode::Add => {
                    let rs1 = self.regs.get(&instruction.rs1);
                    let rs2 = self.regs.get(&instruction.rs2);
                    let imm = instruction.imm as Word;
                    self.regs.set(instruction.rd, rs1 + rs2 + imm);
                }
                Opcode::ECall => match self.regs.get(&RegisterID::A7).try_into()? {
                    Syscall::Write => {
                        let fd = self.regs.get(&RegisterID::A0);
                        assert_eq!(fd, 1, "expected file descriptor to specify stdout (1)");

                        let buf_addr = self.regs.get(&RegisterID::A1);
                        let len = self.regs.get(&RegisterID::A2);

                        for i in 0..len {
                            let word = self.mem.get(buf_addr + i as Address);
                            self.write_byte(word as u8);
                        }
                    }
                },
                Opcode::EBreak => break,
            }
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
enum Syscall {
    Write,
}

impl TryFrom<Word> for Syscall {
    type Error = Error;

    fn try_from(word: Word) -> Result<Self> {
        match word {
            64 => Ok(Syscall::Write),
            _ => Err(Error::SyscallUnknown(word)),
        }
    }
}

#[derive(Debug, PartialEq)]
enum Opcode {
    LoadImmediate,
    Add,
    ECall,
    EBreak,
}

impl TryFrom<Word> for Opcode {
    type Error = Error;

    fn try_from(word: Word) -> Result<Self> {
        match word {
            0b00001 => Ok(Opcode::LoadImmediate),
            0b00010 => Ok(Opcode::Add),
            0b10111 => Ok(Opcode::ECall),
            0b11000 => Ok(Opcode::EBreak),
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
    use claims::{assert_err, assert_err_eq, assert_ok, assert_ok_eq, assert_some_eq};

    #[test]
    fn new_returns_initialized_machine() {
        let want: Machine = Machine {
            pc: 0u32,
            out: vec![],
            mem: Memory::default(),
            regs: Registers::default(),
        };
        let got = Machine::new();
        assert_eq!(want, got);
    }

    #[test]
    fn parsing_an_invalid_syscall_returns_an_error() {
        assert_err_eq!(Syscall::try_from(0), Error::SyscallUnknown(0));
    }

    #[test]
    fn syscalls_can_be_parsed_from_integer_ids() {
        struct TestCase {
            word: Word,
            want: Syscall,
        }
        let cases = [TestCase {
            word: 64,
            want: Syscall::Write,
        }];
        for case in cases {
            assert_ok_eq!(Syscall::try_from(case.word), case.want);
        }
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
            TestCase {
                word: 0b10111,
                want: Opcode::ECall,
            },
            TestCase {
                word: 0b11000,
                want: Opcode::EBreak,
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
            TestCase {
                word: 0b0000_0000_0000_0000_0000_0000_0001_0111,
                want: Instruction {
                    opcode: Opcode::ECall,
                    rd: RegisterID::X0,
                    rs1: RegisterID::X0,
                    rs2: RegisterID::X0,
                    imm: 0,
                },
            },
            TestCase {
                word: 0b0000_0000_0000_0000_0000_0000_0001_1000,
                want: Instruction {
                    opcode: Opcode::EBreak,
                    rd: RegisterID::X0,
                    rs1: RegisterID::X0,
                    rs2: RegisterID::X0,
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
        let mut machine = Machine {
            mem: Memory::from([(0, 0b0000_0000_0000_0100_0000_0000_0010_0001)]),
            ..Default::default()
        };

        machine.run();

        let want = Machine {
            pc: 1,
            out: Vec::new(),
            regs: Registers::from([(RegisterID::A0, 2)]),
            mem: Memory::from([(0, 0b0000_0000_0000_0100_0000_0000_0010_0001)]),
        };
        assert_eq!(want, machine);
    }

    #[test]
    fn run_executes_an_add_instruction() {
        let mut machine = Machine {
            regs: Registers::from([(RegisterID::A1, 2), (RegisterID::A2, 3)]),
            mem: Memory::from([(0, 0b0000_0000_0000_0010_0110_0100_0010_0010)]),
            ..Default::default()
        };

        machine.run();

        let want = Machine {
            pc: 1,
            out: Vec::new(),
            regs: Registers::from([
                (RegisterID::A0, 6),
                (RegisterID::A1, 2),
                (RegisterID::A2, 3),
            ]),
            mem: Memory::from([(0, 0b0000_0000_0000_0010_0110_0100_0010_0010)]),
        };
        assert_eq!(want, machine);
    }

    #[test]
    fn run_executes_an_ebreak_instruction() {
        let mut machine = Machine {
            mem: Memory::from([(0, 0b0000_0000_0000_0000_0000_0000_0001_1000)]),
            ..Default::default()
        };

        assert_ok!(machine.run());

        let want = Machine {
            pc: 1,
            out: Vec::new(),
            regs: Registers::default(),
            mem: Memory::from([(0, 0b0000_0000_0000_0000_0000_0000_0001_1000)]),
        };
        assert_eq!(want, machine);
    }

    #[test]
    fn run_executes_an_ecall_instruction_that_writes_data_to_stdout() {
        // TODO: Implement `add` in a depth-first approach:
        // write a test that can assemble a program from source and execute it.
        /*
        jmp 6
        'h'
        'e'
        'l'
        'l'
        'o'
        li a0, 1  # fd = 1 (stdout)
        la a1, 1
        li a2, 5
        li a7, 64 # write syscall
        ecall
        ebreak
        */
        let mut out = Vec::new();
        let mut machine = Machine {
            pc: 0,
            out,
            regs: Registers::from([
                (RegisterID::A0, 1),  // fd = 1 (stdout)
                (RegisterID::A1, 2),  // *buf = 2
                (RegisterID::A2, 5),  // len = 5
                (RegisterID::A7, 64), // syscall "write"
            ]),
            mem: Memory::from([
                // ECall
                (0, 0b0000_0000_0001_0111),
                // EBreak
                (1, 0b0000_0000_0001_1000),
                // data
                (2, 'h' as Word),
                (3, 'e' as Word),
                (4, 'l' as Word),
                (5, 'l' as Word),
                (6, 'o' as Word),
            ]),
        };
        assert_ok!(machine.run());

        let want = "hello".to_string();
        let got = String::from_utf8(machine.out).unwrap();
        assert_eq!(want, got);
    }

    #[test]
    fn run_executes_multiple_add_instructions() {
        let mut machine = Machine {
            mem: Memory::from([
                // Add
                (0, 0b0000_0000_0000_0010_0000_0010_0010_0010),
                // Add
                (1, 0b0000_0000_0000_0010_0000_0010_0010_0010),
                // Add
                (2, 0b0000_0000_0000_0010_0000_0010_0010_0010),
                // EBreak
                (3, 0b0000_0000_0000_0000_0000_0000_0001_1000),
            ]),
            ..Default::default()
        };
        assert_ok!(machine.run());

        let want = Machine {
            pc: 4,
            out: Vec::new(),
            regs: Registers::from([(RegisterID::A0, 3)]),
            mem: Memory::from([
                // Add
                (0, 0b0000_0000_0000_0010_0000_0010_0010_0010),
                // Add
                (1, 0b0000_0000_0000_0010_0000_0010_0010_0010),
                // Add
                (2, 0b0000_0000_0000_0010_0000_0010_0010_0010),
                // EBreak
                (3, 0b0000_0000_0000_0000_0000_0000_0001_1000),
            ]),
        };
        assert_eq!(want, machine);
    }

    #[test]
    fn x0_register_is_always_zero() {
        let mut registers = Registers::default();

        assert_eq!(registers.get(&RegisterID::X0), 0);

        registers.set(RegisterID::X0, 42);
        assert_eq!(registers.get(&RegisterID::X0), 0);
    }
}
