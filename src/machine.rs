use crate::asm::{Address, Instruction, Opcode, Reg, Word};
use anyhow::{anyhow, bail, Result};
use std::{collections::HashMap, fmt::Display};

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Memory {
    inner: HashMap<Address, Word>,
}

impl Memory {
    pub fn get(&self, addr: Address) -> Word {
        *self.inner.get(&addr).unwrap_or(&Word::default())
    }

    fn set(&mut self, addr: Address, word: Word) {
        self.inner.insert(addr, word);
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
pub struct Registers {
    inner: HashMap<Reg, Word>,
}

impl Registers {
    pub fn get(&self, reg: Reg) -> Word {
        *self.inner.get(&reg).unwrap_or(&Word::default())
    }

    fn set(&mut self, reg: Reg, value: Word) {
        let value = match reg {
            Reg::zero => 0,
            _ => value,
        };
        self.inner.insert(reg, value);
    }
}

impl<const N: usize> From<[(Reg, Word); N]> for Registers {
    fn from(values: [(Reg, Word); N]) -> Self {
        Self {
            inner: HashMap::from(values),
        }
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Machine {
    pub pc: Word,
    pub mem: Memory,
    pub regs: Registers,
    out: Vec<Word>,
}

impl Machine {
    const SYSCALL_WRITE: u32 = 64;
    const FD_STDOUT: u32 = 1;

    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_image(&mut self, image: Vec<Word>) {
        for (i, word) in image.into_iter().enumerate() {
            self.mem.set(i as Address, word);
        }
    }

    fn next(&mut self) -> Result<Instruction> {
        let word = self.mem.get(self.pc);
        Instruction::try_from(word)
    }

    fn write(&mut self, data: Word) {
        self.out.push(data);
    }

    /// Runs the machine until the next breakpoint or error.
    ///
    /// # Errors
    ///
    /// Returns an error if an illegal instruction is encountered, or if an
    /// unknown file descriptor is specified for a syscall.
    pub fn run(&mut self) -> Result<()> {
        loop {
            self.execute_next()?;
        }
    }

    /// Executes the next instruction.
    ///
    /// # Errors
    ///
    /// Returns an error if an illegal instruction is encountered, or if an
    /// unknown file descriptor is specified for a syscall.
    pub fn execute_next(&mut self) -> Result<()> {
        let pc = self.pc;
        let instruction = self.next()?;
        self.pc += 1;

        let opcode = instruction.opcode;
        let rd = instruction.rd;
        let rs1 = self.regs.get(instruction.rs1);
        let _rs2 = self.regs.get(instruction.rs2);
        let imm = instruction.imm;

        match opcode {
            Opcode::unimp => {
                bail!("Illegal instruction at pc={:04x}", self.pc);
            }
            Opcode::addi => {
                self.regs.set(rd, rs1 + imm);
            }
            Opcode::auipc => {
                self.regs.set(rd, pc + (imm << 12));
            }
            Opcode::ecall => {
                let syscall = self.regs.get(Reg::a7);
                match syscall {
                    Machine::SYSCALL_WRITE => {
                        let fd = self.regs.get(Reg::a0);
                        let buf = self.regs.get(Reg::a1);
                        let count = self.regs.get(Reg::a2);

                        for i in 0..count {
                            let c = self.mem.get(buf + i);
                            match fd {
                                Machine::FD_STDOUT => self.out.push(c),

                                _ => bail!("Unknown fd: {fd:04x} at pc={:04x}", self.pc),
                            }
                        }
                    }
                    _ => todo!(),
                }
            }
            Opcode::lui => {
                self.regs.set(rd, imm << 12);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::asm;

    use super::*;
    use claims::{assert_err, assert_err_eq, assert_ok, assert_ok_eq, assert_some_eq};

    #[test]
    fn decodes_and_encodes_instructions_successfully() {
        struct TestCase {
            word: Word,
            instruction: Instruction,
        }
        let cases = vec![
            TestCase {
                // I-Type:
                //      iiii_iiii_iiii_ssss_sfff_dddd_dooo_oooo
                word: 0b0000_0010_0000_0101_1000_0101_1001_0011,
                instruction: Instruction {
                    opcode: Opcode::addi,
                    rd: Reg::a1,
                    rs1: Reg::a1,
                    rs2: Reg::zero,
                    imm: 32,
                },
            },
            TestCase {
                // U-Type:
                //      iiii_iiii_iiii_iiii_iiii_dddd_dooo_oooo
                word: 0b0000_0000_0000_0000_0010_0101_0001_0111,
                instruction: Instruction {
                    opcode: Opcode::auipc,
                    rd: Reg::a0,
                    rs1: Reg::zero,
                    rs2: Reg::zero,
                    imm: 2,
                },
            },
            TestCase {
                word: 0b0000_0000_0000_0000_0000_0000_0111_0011,
                instruction: Instruction {
                    opcode: Opcode::ecall,
                    rd: Reg::zero,
                    rs1: Reg::zero,
                    rs2: Reg::zero,
                    imm: 0,
                },
            },
            TestCase {
                // U-Type:
                //      iiii_iiii_iiii_iiii_iiii_dddd_dooo_oooo
                word: 0b0000_0000_0000_0000_0010_0101_0011_0111,
                instruction: Instruction {
                    opcode: Opcode::lui,
                    rd: Reg::a0,
                    rs1: Reg::zero,
                    rs2: Reg::zero,
                    imm: 2,
                },
            },
        ];

        for case in cases {
            let got = Instruction::try_from(case.word).unwrap();
            assert_eq!(
                case.instruction, got,
                "failed to decode instruction from word"
            );

            let got: Word = got.into();

            assert_eq!(
                case.word, got,
                "failed to encode instruction into word: {:b}, {:b}",
                case.word, got,
            );
        }
    }

    #[test]
    fn executes_lui_instruction_successfully() {
        let mut machine = Machine::default();

        let instruction = Instruction {
            opcode: Opcode::lui,
            rd: Reg::a0,
            imm: 2,
            rs1: Reg::zero,
            rs2: Reg::zero,
        };
        machine.mem.set(0, instruction.into());

        machine.run();

        let want = 2 << 12;
        let got = machine.regs.get(Reg::a0);
        assert_eq!(want, got);
    }

    #[test]
    fn executes_auipc_instruction_successfully() {
        let mut machine = Machine::default();

        let instruction = Instruction {
            opcode: Opcode::auipc,
            rd: Reg::a0,
            rs1: Reg::zero,
            rs2: Reg::zero,
            imm: 2,
        };
        machine.mem.set(0, instruction.into());

        machine.run();

        let want = 2 << 12;
        let got = machine.regs.get(Reg::a0);
        assert_eq!(want, got);
    }

    #[test]
    fn executes_addi_instruction_successfully() {
        let mut machine = Machine::default();

        let instruction = Instruction {
            opcode: Opcode::addi,
            rd: Reg::a0,
            rs1: Reg::zero,
            rs2: Reg::zero,
            imm: 2,
        };
        machine.mem.set(0, instruction.into());

        machine.run();

        let want = 2;
        let got = machine.regs.get(Reg::a0);
        assert_eq!(want, got);
    }

    #[test]
    fn executes_ecall_instruction_successfully() {
        // .section .text
        // _start:
        //   li a0, 1  # fd = 1 (stdout)
        //   la a1, helloworld
        //   li a2, 13
        //   li a7, 64 # write syscall
        //   ecall
        // helloworld:
        //   .ascii "Hello World!\n"

        let mut machine = Machine::default();

        let instructons = [
            Instruction {
                opcode: Opcode::addi,
                rd: Reg::a0,
                rs1: Reg::zero,
                rs2: Reg::zero,
                imm: 1,
            },
            Instruction {
                opcode: Opcode::auipc,
                rd: Reg::a1,
                rs1: Reg::zero,
                rs2: Reg::zero,
                imm: 0,
            },
            Instruction {
                opcode: Opcode::addi,
                rd: Reg::a1,
                rs1: Reg::a1,
                rs2: Reg::zero,
                imm: 5,
            },
            Instruction {
                opcode: Opcode::addi,
                rd: Reg::a2,
                rs1: Reg::zero,
                rs2: Reg::zero,
                imm: 13,
            },
            Instruction {
                opcode: Opcode::addi,
                rd: Reg::a7,
                rs1: Reg::zero,
                rs2: Reg::zero,
                imm: 64,
            },
            Instruction {
                opcode: Opcode::ecall,
                rd: Reg::zero,
                rs1: Reg::zero,
                rs2: Reg::zero,
                imm: 0,
            },
        ];
        let len = instructons.len();
        for (i, instruction) in instructons.into_iter().enumerate() {
            machine.mem.set(i as Address, instruction.into());
        }
        let hello_world: Vec<Word> = "Hello World!\n".chars().map(|c| c as Word).collect();
        for (i, c) in hello_world.iter().enumerate() {
            machine.mem.set((i + len) as Address, *c);
        }

        assert_err!(machine.run());

        let got = machine.out;
        assert_eq!(got, hello_world);
    }

    #[test]
    fn add_immediate_1() {
        let image = asm::assemble("li a0, 1").unwrap();

        let mut machine = Machine::default();
        machine.load_image(image);

        assert_err!(machine.run());

        let want = 1;
        let got = machine.regs.get(Reg::a0);
        assert_eq!(want, got, "wrong a0: {want}, expected: {got}");
    }
}
