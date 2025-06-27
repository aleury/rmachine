use crate::asm::{Address, Instruction, Opcode, Reg, Word};
use anyhow::{anyhow, bail, Result};
use std::{
    char::REPLACEMENT_CHARACTER,
    collections::HashMap,
    fmt::Display,
    io::{stdout, Stdout, Write},
};

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
            data.push(self.get(addr + (offset * 4) as Word));
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

pub trait Sys: Write {}

pub struct TermSys {
    out: Stdout,
}

impl Default for TermSys {
    fn default() -> Self {
        Self { out: stdout() }
    }
}

impl Write for &mut TermSys {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.out.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.out.flush()
    }
}

impl Sys for &mut TermSys {}

impl TermSys {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Machine {
    pub pc: Word,
    pub mem: Memory,
    pub regs: Registers,
}

impl Machine {
    const SYSCALL_WRITE: u32 = 64;
    const SYSCALL_EXIT: u32 = 93;
    const FD_STDOUT: u32 = 1;

    #[must_use]
    pub fn new() -> Self {
        Self {
            pc: Word::default(),
            mem: Memory::default(),
            regs: Registers::default(),
        }
    }

    pub fn load_image(&mut self, image: &[Word]) {
        for (i, word) in image.iter().enumerate() {
            self.mem.set((i * size_of::<Word>()) as Address, *word);
        }
    }

    fn next(&mut self) -> Instruction {
        let word = self.mem.get(self.pc);
        Instruction::from(word)
    }

    /// Runs the machine until the next breakpoint or error.
    ///
    /// # Errors
    ///
    /// Returns an error if an illegal instruction is encountered, or if an
    /// unknown file descriptor is specified for a syscall.
    pub fn run(&mut self, mut sys: impl Sys) -> Result<()> {
        let mysys = &mut sys;
        loop {
            self.execute_next(mysys)?;
        }
    }

    /// Executes the next instruction.
    ///
    /// # Errors
    ///
    /// Returns an error if an illegal instruction is encountered, or if an
    /// unknown file descriptor is specified for a syscall.
    pub fn execute_next(&mut self, mut sys: &mut impl Sys) -> Result<()> {
        let pc = self.pc;
        let instruction = self.next();
        self.pc += size_of::<Word>() as u32;

        let opcode = instruction.opcode;
        let rd = instruction.rd;
        let rs1 = self.regs.get(instruction.rs1);
        let rs2 = self.regs.get(instruction.rs2);
        let imm = instruction.imm;

        match opcode {
            Opcode::add => {
                self.regs.set(rd, rs1 + rs2);
            }
            Opcode::addi => {
                self.regs.set(rd, rs1 + imm);
            }
            Opcode::auipc => {
                self.regs.set(rd, pc + (imm << 12));
            }
            Opcode::beq => {
                todo!("implement beq")
            }
            Opcode::ecall => match self.regs.get(Reg::a7) {
                Self::SYSCALL_WRITE => {
                    let fd = self.regs.get(Reg::a0);
                    let buf = self.regs.get(Reg::a1);
                    let len = self.regs.get(Reg::a2);
                    let word_size = size_of::<Word>() as Address;

                    let words = self.mem.read(buf, len as usize);
                    let string: String = words
                        .into_iter()
                        .map(|w| char::from_u32(w).unwrap_or(REPLACEMENT_CHARACTER))
                        .collect();
                    write!(sys, "{string}")?;
                }
                Self::SYSCALL_EXIT => {
                    let code = self.regs.get(Reg::a0);
                    std::process::exit(code.try_into().unwrap_or_else(|err| {
                        eprintln!("Exit code out of range: {err}");
                        1
                    }));
                }
                _ => todo!(),
            },
            Opcode::jal => {
                todo!("implement jal")
            }
            Opcode::lb => {
                let addr = rs1 + imm;
                let value = self.mem.get(addr);
                self.regs.set(rd, value);
            }
            Opcode::lui => {
                self.regs.set(rd, imm << 12);
            }
            Opcode::unimp => {
                bail!("Illegal instruction at pc={pc:04x}");
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

    struct TestSys {
        out: Vec<u8>,
    }

    impl Write for &mut TestSys {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.out.write(buf)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl Sys for &mut TestSys {}

    impl TestSys {
        fn new() -> Self {
            Self { out: Vec::new() }
        }
    }

    #[test]
    fn load_image_loads_image_into_machine() {
        let bytes = vec![99];
        let mut machine = Machine::new();
        machine.load_image(&bytes);
        let got = machine.mem.get(0);
        assert_eq!(got, 99);
    }

    #[test]
    fn executes_lui_instruction_successfully() {
        let mut machine = Machine::new();

        let instruction = Instruction {
            opcode: Opcode::lui,
            rd: Reg::a0,
            imm: 2,
            rs1: Reg::zero,
            rs2: Reg::zero,
        };
        machine.mem.set(0, instruction.into());

        machine.run(&mut TestSys::new());

        let want = 2 << 12;
        let got = machine.regs.get(Reg::a0);
        assert_eq!(want, got);
    }

    #[test]
    fn executes_auipc_instruction_successfully() {
        let mut machine = Machine::new();

        let instruction = Instruction {
            opcode: Opcode::auipc,
            rd: Reg::a0,
            rs1: Reg::zero,
            rs2: Reg::zero,
            imm: 2,
        };
        machine.mem.set(0, instruction.into());

        machine.run(&mut TestSys::new());

        let want = 2 << 12;
        let got = machine.regs.get(Reg::a0);
        assert_eq!(want, got);
    }

    #[test]
    fn executes_addi_instruction_successfully() {
        let mut machine = Machine::new();

        let instruction = Instruction {
            opcode: Opcode::addi,
            rd: Reg::a0,
            rs1: Reg::zero,
            rs2: Reg::zero,
            imm: 2,
        };
        machine.mem.set(0, instruction.into());

        machine.run(&mut TestSys::new());

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

        let mut machine = Machine::new();

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

        let offset = len * word_size;
        let hello_world: Vec<Word> = "Hello World!\n".chars().map(|c| c as Word).collect();
        for (i, c) in hello_world.iter().enumerate() {
            let addr = offset + (i * word_size);
            machine.mem.set(addr as Address, *c);
        }
        let mut sys = TestSys::new();
        assert_err!(machine.run(&mut sys));

        assert_eq!(sys.out, b"Hello World!\n");
    }

    #[test]
    fn add_immediate_1() {
        let image = asm::assemble("li a0, 1").unwrap();

        let mut machine = Machine::new();
        machine.load_image(&image);

        assert_err!(machine.run(&mut TestSys::new()));

        let want = 1;
        let got = machine.regs.get(Reg::a0);
        assert_eq!(want, got, "wrong a0: {want}, expected: {got}");
    }

    #[test]
    #[ignore]
    fn strlen_program_counts_characters_in_a_string() {
        let program = r#"
            la     a0, mystr     # Load the address of mystr into a0
            li     t0, 0         # i = 0
        loop: # Start of for loop
            add    t1, t0, a0    # Add the byte offset for str[i]
            lb     t1, 0(t1)     # Dereference str[i]
            beq    t1, zero, end # if str[i] == 0, break for loop
            addi   t0, t0, 1     # Add 1 to our iterator
            add    t1, t0, a0
            j      loop          # Jump back to condition (1 backwards)
        end: # End of for loop
            ebreak

        mystr:
            .ascii "test\0"
        "#;

        let image = asm::assemble(program).unwrap();

        let mut machine = Machine::new();
        machine.load_image(&image);
    }
}
