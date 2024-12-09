#![allow(
    unused,
    clippy::cast_possible_truncation,
    clippy::needless_pass_by_value
)]
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
enum Error {
    OpcodeUnknown(u32),
    RegisterUnknown(u32),
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
struct Registers {
    inner: HashMap<Reg, Word>,
}

impl Registers {
    fn get(&self, reg: Reg) -> Word {
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
struct Machine {
    pc: Word,
    mem: Memory,
    regs: Registers,
    out: Vec<u8>,
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

            let opcode = instruction.opcode;
            let rd = instruction.rd;
            let rs1 = self.regs.get(instruction.rs1);
            let rs2 = self.regs.get(instruction.rs2);
            let imm = instruction.imm;

            match opcode {
                Opcode::AddImmediate => {
                    self.regs.set(rd, rs1 + imm);
                }
                Opcode::AddUpperImmediateToProgramCounter => {
                    self.regs.set(rd, self.pc + (imm << 12));
                }
                Opcode::EnvironmentCall => todo!(),
                Opcode::LoadUpperImmediate => {
                    self.regs.set(rd, imm << 12);
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq)]
enum Opcode {
    #[default]
    AddImmediate,
    AddUpperImmediateToProgramCounter,
    EnvironmentCall,
    LoadUpperImmediate,
}

impl TryFrom<Word> for Opcode {
    type Error = Error;

    fn try_from(word: Word) -> Result<Self> {
        match word {
            0b001_0011 => Ok(Opcode::AddImmediate),
            0b001_0111 => Ok(Opcode::AddUpperImmediateToProgramCounter),
            0b111_0011 => Ok(Opcode::EnvironmentCall),
            0b011_0111 => Ok(Opcode::LoadUpperImmediate),
            _ => Err(Error::OpcodeUnknown(word)),
        }
    }
}

impl From<Opcode> for Word {
    fn from(value: Opcode) -> Self {
        match value {
            Opcode::AddImmediate => 0b001_0011,
            Opcode::AddUpperImmediateToProgramCounter => 0b001_0111,
            Opcode::EnvironmentCall => 0b111_0011,
            Opcode::LoadUpperImmediate => 0b011_0111,
        }
    }
}

// x0 zero Hard-wired zero —
// x1 ra Return address Caller
// x2 sp Stack pointer Callee
// x3 gp Global pointer —
// x4 tp Thread pointer —
// x5 t0 Temporary/alternate link register Caller
// x6–7 t1–2 Temporaries Caller
// x8 s0/fp Saved register/frame pointer Callee
// x9 s1 Saved register Callee
// x10–11 a0–1 Function arguments/return values Caller
// x12–17 a2–7 Function arguments Caller
// x18–27 s2–11 Saved registers Callee
// x28–31 t3–6 Temporaries Caller
// f0–7 ft0–7 FP temporaries Caller
// f8–9 fs0–1 FP saved registers Callee
// f10–11 fa0–1 FP arguments/return values Caller
// f12–17 fa2–7 FP arguments Caller
// f18–27 fs2–11 FP saved registers Callee
// f28–31 ft8–11 FP temporaries Caller

#[allow(non_camel_case_types)]
#[derive(Debug, Default, Eq, PartialEq, Hash, PartialOrd)]
enum Reg {
    #[default]
    zero,
    ra,
    sp,
    gp,
    tp,
    t0,
    t1,
    t2,
    s0,
    s1,
    a0,
    a1,
    a2,
    a3,
    a4,
    a5,
}

impl TryFrom<Word> for Reg {
    type Error = Error;

    fn try_from(word: Word) -> Result<Self> {
        match word {
            0b00000 => Ok(Reg::zero),
            0b01010 => Ok(Reg::a0),
            _ => Err(Error::RegisterUnknown(word)),
        }
    }
}

impl From<Reg> for Word {
    fn from(register_id: Reg) -> Self {
        match register_id {
            Reg::zero => 0b00000,
            Reg::a0 => 0b01010,
            _ => todo!(),
        }
    }
}

#[derive(Debug, PartialEq, Default)]
struct Instruction {
    opcode: Opcode,
    rd: Reg,
    rs1: Reg,
    rs2: Reg,
    imm: u32,
}

impl TryFrom<Word> for Instruction {
    type Error = Error;

    fn try_from(word: Word) -> Result<Self> {
        let opcode = (word & 0b0111_1111).try_into()?;
        match opcode {
            Opcode::AddImmediate => {
                let rd = ((word >> 7) & 0b0001_1111).try_into()?;
                let rs1 = ((word >> 11) & 0b0001_1111).try_into()?;
                let imm = (word >> 20);
                Ok(Instruction {
                    opcode,
                    rd,
                    rs1,
                    imm,
                    ..Default::default()
                })
            }
            Opcode::AddUpperImmediateToProgramCounter => {
                let rd = ((word >> 7) & 0b0001_1111).try_into()?;
                let imm = (word >> 12);
                Ok(Instruction {
                    opcode,
                    rd,
                    imm,
                    ..Default::default()
                })
            }
            Opcode::EnvironmentCall => Ok(Instruction {
                opcode,
                ..Default::default()
            }),
            Opcode::LoadUpperImmediate => {
                let rd = ((word >> 7) & 0b0001_1111).try_into()?;
                let imm = (word >> 12);
                Ok(Instruction {
                    opcode,
                    rd,
                    imm,
                    ..Default::default()
                })
            }
        }
    }
}

impl From<Instruction> for Word {
    fn from(instruction: Instruction) -> Self {
        match instruction.opcode {
            Opcode::AddImmediate => {
                let opcode: Word = instruction.opcode.into();
                let rd: Word = instruction.rd.into();
                let f3: Word = 0b000;
                let rs1: Word = instruction.rs1.into();
                let imm: Word = instruction.imm;

                opcode | (rd << 7) | (f3 << 11) | (rs1 << 14) | (imm << 20)
            }
            Opcode::AddUpperImmediateToProgramCounter | Opcode::LoadUpperImmediate => {
                let opcode: Word = instruction.opcode.into();
                let rd: Word = instruction.rd.into();
                let imm: Word = instruction.imm;

                opcode | (rd << 7) | (imm << 12)
            }
            Opcode::EnvironmentCall => instruction.opcode.into(),
        }
    }
}

#[cfg(test)]
mod tests {
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
                word: 0b0000_0000_0010_0000_0000_0101_0001_0011,
                instruction: Instruction {
                    opcode: Opcode::AddImmediate,
                    rd: Reg::a0,
                    rs1: Reg::zero,
                    rs2: Reg::zero,
                    imm: 2,
                },
            },
            TestCase {
                // U-Type:
                //      iiii_iiii_iiii_iiii_iiii_dddd_dooo_oooo
                word: 0b0000_0000_0000_0000_0010_0101_0001_0111,
                instruction: Instruction {
                    opcode: Opcode::AddUpperImmediateToProgramCounter,
                    rd: Reg::a0,
                    rs1: Reg::zero,
                    rs2: Reg::zero,
                    imm: 2,
                },
            },
            TestCase {
                word: 0b0000_0000_0000_0000_0000_0000_0111_0011,
                instruction: Instruction {
                    opcode: Opcode::EnvironmentCall,
                    ..Default::default()
                },
            },
            TestCase {
                // U-Type:
                //      iiii_iiii_iiii_iiii_iiii_dddd_dooo_oooo
                word: 0b0000_0000_0000_0000_0010_0101_0011_0111,
                instruction: Instruction {
                    opcode: Opcode::LoadUpperImmediate,
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
            opcode: Opcode::LoadUpperImmediate,
            rd: Reg::a0,
            imm: 2,
            ..Default::default()
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
            opcode: Opcode::AddUpperImmediateToProgramCounter,
            rd: Reg::a0,
            imm: 2,
            ..Default::default()
        };
        machine.mem.set(0, instruction.into());

        machine.run();

        let want = 1 + (2 << 12);
        let got = machine.regs.get(Reg::a0);
        assert_eq!(want, got);
    }

    #[test]
    fn executes_addi_instruction_successfully() {
        let mut machine = Machine::default();

        let instruction = Instruction {
            opcode: Opcode::AddImmediate,
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
}
