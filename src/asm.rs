use anyhow::{anyhow, Ok, Result};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fmt::Display;
use std::path::Path;

use crate::ast::{self, Directive, Line, Operand};
use crate::lexer;
use crate::parser::Parser;

pub type Word = u32;

pub type Address = u32;

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
pub enum Reg {
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
    a6,
    a7,
}

impl Display for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Reg::zero => "zero",
                Reg::ra => "ra",
                Reg::sp => "sp",
                Reg::gp => "gp",
                Reg::tp => "tp",
                Reg::t0 => "t0",
                Reg::t1 => "t1",
                Reg::t2 => "t2",
                Reg::s0 => "s0",
                Reg::s1 => "s1",
                Reg::a0 => "a0",
                Reg::a1 => "a1",
                Reg::a2 => "a2",
                Reg::a3 => "a3",
                Reg::a4 => "a4",
                Reg::a5 => "a5",
                Reg::a6 => "a6",
                Reg::a7 => "a7",
            }
        )
    }
}

impl From<Word> for Reg {
    fn from(word: Word) -> Self {
        match word {
            0b00000 => Reg::zero,
            0b01010 => Reg::a0,
            0b01011 => Reg::a1,
            0b01100 => Reg::a2,
            0b10001 => Reg::a7,
            _ => panic!("unknown register: {word:#?}"),
        }
    }
}

impl From<Reg> for Word {
    fn from(register_id: Reg) -> Self {
        match register_id {
            Reg::zero => 0b00000,
            Reg::a0 => 0b01010,
            Reg::a1 => 0b01011,
            Reg::a2 => 0b01100,
            Reg::a7 => 0b10001,
            _ => todo!(),
        }
    }
}

impl TryFrom<String> for Reg {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "zero" => Ok(Reg::zero),
            "a0" => Ok(Reg::a0),
            "a1" => Ok(Reg::a1),
            "a2" => Ok(Reg::a2),
            "a3" => Ok(Reg::a3),
            "a4" => Ok(Reg::a4),
            "a5" => Ok(Reg::a5),
            "a6" => Ok(Reg::a6),
            "a7" => Ok(Reg::a7),
            _ => Err(anyhow!("unknown register: {value:#?}")),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Eq, PartialEq, Hash, PartialOrd)]
pub enum Opcode {
    unimp,
    addi,
    auipc,
    lui,
    ecall,
}

impl Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:6}",
            match self {
                Opcode::unimp => "unimp",
                Opcode::addi => "addi",
                Opcode::auipc => "auipc",
                Opcode::lui => "lui",
                Opcode::ecall => "ecall",
            }
        )
    }
}

impl From<Word> for Opcode {
    fn from(word: Word) -> Self {
        match word {
            0b001_0011 => Opcode::addi,
            0b001_0111 => Opcode::auipc,
            0b111_0011 => Opcode::ecall,
            0b011_0111 => Opcode::lui,
            _ => Opcode::unimp,
        }
    }
}

impl From<Opcode> for Word {
    fn from(value: Opcode) -> Self {
        match value {
            Opcode::unimp => 0b000_0000,
            Opcode::addi => 0b001_0011,
            Opcode::auipc => 0b001_0111,
            Opcode::ecall => 0b111_0011,
            Opcode::lui => 0b011_0111,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Instruction {
    pub opcode: Opcode,
    pub rd: Reg,
    pub rs1: Reg,
    pub rs2: Reg,
    pub imm: u32,
}

impl Instruction {
    const RD: u32 = 7;

    const I_F3: u32 = 12;
    const I_RS1: u32 = 15;
    const I_IMM: u32 = 20;

    const U_IMM: u32 = 12;

    const OP_MASK: u32 = 0b0111_1111;
    const R_MASK: u32 = 0b0001_1111;
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.opcode {
            Opcode::unimp | Opcode::ecall => write!(f, "{}", self.opcode),
            Opcode::auipc => write!(f, "{} {}, 0x{:02x}", self.opcode, self.rd, self.imm),
            _ => write!(
                f,
                "{} {}, {}, {}, 0x{:02x}",
                self.opcode, self.rd, self.rs1, self.rs2, self.imm
            ),
        }
    }
}

impl From<Word> for Instruction {
    fn from(word: Word) -> Self {
        let opcode = (word & Instruction::OP_MASK).into();
        match opcode {
            Opcode::unimp | Opcode::ecall => Instruction {
                opcode,
                rd: Reg::zero,
                rs1: Reg::zero,
                rs2: Reg::zero,
                imm: 0,
            },
            Opcode::addi => {
                let rd = ((word >> Instruction::RD) & Instruction::R_MASK).into();
                let rs1 = ((word >> Instruction::I_RS1) & Instruction::R_MASK).into();
                let imm = word >> Instruction::I_IMM;
                Instruction {
                    opcode,
                    rd,
                    rs1,
                    rs2: Reg::zero,
                    imm,
                }
            }
            Opcode::auipc => {
                let rd = ((word >> Instruction::RD) & Instruction::R_MASK).into();
                let imm = word >> Instruction::U_IMM;
                Instruction {
                    opcode,
                    rd,
                    rs1: Reg::zero,
                    rs2: Reg::zero,
                    imm,
                }
            }
            Opcode::lui => {
                let rd = ((word >> Instruction::RD) & Instruction::R_MASK).into();
                let imm = word >> Instruction::U_IMM;
                Instruction {
                    opcode,
                    rd,
                    imm,
                    rs1: Reg::zero,
                    rs2: Reg::zero,
                }
            }
        }
    }
}

impl From<Instruction> for Word {
    fn from(instruction: Instruction) -> Self {
        match instruction.opcode {
            Opcode::unimp => 0,
            Opcode::addi => {
                let opcode: Word = instruction.opcode.into();
                let rd: Word = instruction.rd.into();
                let f3: Word = 0b000;
                let rs1: Word = instruction.rs1.into();
                let imm: Word = instruction.imm;

                opcode
                    | (rd << Instruction::RD)
                    | (f3 << Instruction::I_F3)
                    | (rs1 << Instruction::I_RS1)
                    | (imm << Instruction::I_IMM)
            }
            Opcode::auipc | Opcode::lui => {
                let opcode: Word = instruction.opcode.into();
                let rd: Word = instruction.rd.into();
                let imm: Word = instruction.imm;

                opcode | (rd << Instruction::RD) | (imm << Instruction::U_IMM)
            }
            Opcode::ecall => instruction.opcode.into(),
        }
    }
}

struct SymbolTable {
    labels: HashMap<String, Address>,
}

impl SymbolTable {
    fn new() -> Self {
        Self {
            labels: HashMap::new(),
        }
    }

    fn add_label(&mut self, name: impl Into<String>, address: Address) {
        self.labels.insert(name.into(), address);
    }

    fn lookup(&self, name: &str) -> Option<Address> {
        self.labels.get(name).copied()
    }
}

struct Ref {
    name: String,
    address: Address,
}

fn assemble_instruction(
    instr: ast::Instruction,
    address: Address,
    refs: &mut Vec<Ref>,
    symbols: &mut SymbolTable,
) -> Result<Vec<Instruction>> {
    let instructions = match instr.name.as_ref() {
        "la" => {
            assert_eq!(instr.operands.len(), 2, "expected 2 operands for la");
            let Operand::Register(ref rd) = instr.operands[0] else {
                return Err(anyhow!("expected register"));
            };
            let Operand::Symbol(ref symbol) = instr.operands[1] else {
                return Err(anyhow!("expected symbol"));
            };
            refs.push(Ref {
                name: symbol.to_string(),
                address,
            });
            vec![Instruction {
                opcode: Opcode::addi,
                rd: Reg::try_from(rd.to_string())?,
                rs1: Reg::zero,
                rs2: Reg::zero,
                imm: 0,
            }]
        }
        "li" => {
            assert_eq!(instr.operands.len(), 2, "expected 2 operands for li");
            let Operand::Register(ref rd) = instr.operands[0] else {
                return Err(anyhow!("expected register"));
            };
            let Operand::Immediate(imm) = instr.operands[1] else {
                return Err(anyhow!("expected immediate"));
            };
            vec![Instruction {
                opcode: Opcode::addi,
                rd: Reg::try_from(rd.to_string())?,
                rs1: Reg::zero,
                rs2: Reg::zero,
                imm,
            }]
        }
        "ecall" => {
            assert_eq!(instr.operands.len(), 0, "expected 0 operands for ecall");
            vec![Instruction {
                opcode: Opcode::ecall,
                rd: Reg::zero,
                rs1: Reg::zero,
                rs2: Reg::zero,
                imm: 0,
            }]
        }
        _ => todo!(),
    };

    Ok(instructions)
}

#[derive(Debug, PartialEq)]
struct Object {
    data: Vec<Word>,
    instructions: Vec<Instruction>,
}

fn assemble_program(program: ast::Program) -> Result<Object> {
    let mut refs: Vec<Ref> = Vec::new();
    let mut symbols = SymbolTable::new();
    let mut data: Vec<Word> = Vec::new();
    let mut instructions: Vec<Instruction> = Vec::new();

    for line in program.lines {
        let address = instructions.len() as Address;
        match line {
            Line::Label(label) => symbols.add_label(label, address),
            Line::Directive(directive) => match directive {
                Directive::Ascii(string) => {
                    for c in string.chars() {
                        data.push(c as Word);
                    }
                }
                _ => todo!(),
            },
            Line::Instruction(instruction) => {
                let mut instruction =
                    assemble_instruction(instruction, address, &mut refs, &mut symbols)?;
                instructions.append(&mut instruction);
            }
        }
    }

    // Resolve references
    for r in refs {
        instructions[r.address as usize].imm = symbols
            .lookup(&r.name)
            .ok_or(anyhow!("unknown identifier: {:#?}", r.name))?;
    }

    Ok(Object { data, instructions })
}

/// Assembles `input`.
///
/// # Errors
///
/// Returns any errors parsing the input.
pub fn assemble(input: &str) -> Result<Vec<Word>> {
    let tokens = lexer::tokenize(input);
    let mut parser = Parser::new(tokens);

    let program = parser.parse()?;
    let obj = assemble_program(program)?;

    Ok(obj.instructions.into_iter().map(Word::from).collect())
}

/// Builds an executable from `input`.
///
/// # Errors
///
/// Returns any errors reading the input, assembling the program
/// or writing the executable to disk.
pub fn build_exe<I>(input: I, output: I) -> Result<()>
where
    I: AsRef<Path>,
{
    const HEADER: &[u8] = b"rme1";

    let source = std::fs::read_to_string(input)?;
    let program = assemble(&source)?;
    let mut bytes = Vec::from(HEADER);
    bytes.extend_from_slice(
        &program
            .into_iter()
            .flat_map(u32::to_be_bytes)
            .collect::<Vec<u8>>(),
    );
    std::fs::write(output, bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn build_exe_fn_creates_an_executable_from_an_asm_source_file() {
        let mut dir = tempdir().unwrap();
        let mut exe_path = dir.path().to_owned();
        exe_path.push("test");

        build_exe("testdata/hello.s".into(), exe_path.clone()).unwrap();

        let want = vec![b'r', b'm', b'e', b'1', 0, 16, 5, 19];
        let got = std::fs::read(exe_path).unwrap();
        assert_eq!(want, got, "wrong bytes");
    }

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
            let got = Instruction::from(case.word);
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
    fn test_assemble_program_returns_object() {
        let program = parse(
            "_start:
                la a0, helloworld
            helloworld:
                .ascii \"Hello World!\n\"
            ",
        );

        let want = Object {
            data: "Hello World!\n".chars().map(|c| c as Word).collect(),
            instructions: vec![Instruction {
                opcode: Opcode::addi,
                rd: Reg::a0,
                rs1: Reg::zero,
                rs2: Reg::zero,
                imm: 1,
            }],
        };

        let got = assemble_program(program).unwrap();
        assert_eq!(want, got);
    }

    #[test]
    fn test_assemble() {
        struct TestCase {
            program: String,
            want: Instruction,
        }

        let cases = [
            TestCase {
                program: "li a0, 1".into(),
                want: Instruction {
                    opcode: Opcode::addi,
                    rd: Reg::a0,
                    rs1: Reg::zero,
                    rs2: Reg::zero,
                    imm: 1,
                },
            },
            TestCase {
                program: "li a1, 2".into(),
                want: Instruction {
                    opcode: Opcode::addi,
                    rd: Reg::a1,
                    rs1: Reg::zero,
                    rs2: Reg::zero,
                    imm: 2,
                },
            },
            TestCase {
                program: "li a2, 42".into(),
                want: Instruction {
                    opcode: Opcode::addi,
                    rd: Reg::a2,
                    rs1: Reg::zero,
                    rs2: Reg::zero,
                    imm: 42,
                },
            },
            TestCase {
                program: "li a7, 64".into(),
                want: Instruction {
                    opcode: Opcode::addi,
                    rd: Reg::a7,
                    rs1: Reg::zero,
                    rs2: Reg::zero,
                    imm: 64,
                },
            },
            TestCase {
                program: "ecall".into(),
                want: Instruction {
                    opcode: Opcode::ecall,
                    rd: Reg::zero,
                    rs1: Reg::zero,
                    rs2: Reg::zero,
                    imm: 0,
                },
            },
        ];

        for case in cases {
            let want: Vec<Word> = vec![case.want.into()];
            let got = assemble(&case.program).unwrap();
            assert_eq!(want, got);
        }
    }

    fn parse(input: &str) -> ast::Program {
        let tokens = lexer::tokenize(input);
        let mut parser = Parser::new(tokens);
        parser.parse().unwrap()
    }
}
