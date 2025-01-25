use crate::asm::{Address, Instruction, Opcode, Reg, Word};
use anyhow::{anyhow, bail, Result};
use std::{collections::HashMap, fmt::Display, ops::Deref};

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

    pub fn load_image_from_bytes(&mut self, bytes: &[u8]) {
        for (i, chunk) in bytes[4..].chunks(4).enumerate() {
            let word = Word::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
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
        self.pc += 4;

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
    use super::*;
    use crate::asm;
    use claims::assert_err;
    use tempfile::tempdir;

    #[test]
    fn load_image_from_bytes_loads_program_into_machine() {
        let bytes = vec![b'r', b'm', b'e', b'1', 0, 16, 5, 19];
        let mut machine = Machine::new();
        machine.load_image_from_bytes(&bytes);

        let word = Word::from(Instruction {
            opcode: Opcode::addi,
            rd: Reg::a0,
            rs1: Reg::zero,
            rs2: Reg::zero,
            imm: 1,
        });

        let got = machine.mem.get(0);
        assert_eq!(got, 1049875);
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
        // .globl _start
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
                imm: 20,
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
        let word_size = size_of::<Word>();
        for (i, instruction) in instructons.into_iter().enumerate() {
            machine
                .mem
                .set((i * word_size) as Address, instruction.into());
        }
        let hello_world: Vec<Word> = "Hello World!\n".chars().map(|c| c as Word).collect();
        for (i, c) in hello_world.iter().enumerate() {
            machine.mem.set((i + len * word_size) as Address, *c);
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
