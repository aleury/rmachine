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
            Reg::t0 => 0b00101,
            Reg::t1 => 0b00110,
            Reg::a0 => 0b01010,
            Reg::a1 => 0b01011,
            Reg::a2 => 0b01100,
            Reg::a7 => 0b10001,
            _ => todo!("Implement From<Reg>: {register_id}"),
        }
    }
}

impl TryFrom<String> for Reg {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "zero" => Ok(Reg::zero),
            "t0" => Ok(Reg::t0),
            "t1" => Ok(Reg::t1),
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
    add,
    addi,
    auipc,
    ecall,
    lb,
    lui,
    unimp,
}

impl Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:6}",
            match self {
                Opcode::add => "add",
                Opcode::addi => "addi",
                Opcode::auipc => "auipc",
                Opcode::ecall => "ecall",
                Opcode::lb => "lb",
                Opcode::lui => "lui",
                Opcode::unimp => "unimp",
            }
        )
    }
}

impl From<Word> for Opcode {
    fn from(word: Word) -> Self {
        match word {
            0b011_0011 => Opcode::add,
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
            Opcode::add => 0b011_0011,
            Opcode::addi => 0b001_0011,
            Opcode::auipc => 0b001_0111,
            Opcode::ecall => 0b111_0011,
            Opcode::lb => 0b000_0011,
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
    // funct offsets
    const F3: u32 = 12;
    const R_F7: u32 = 25;

    // register offsets
    const RD: u32 = 7;
    const RS1: u32 = 15;
    const RS2: u32 = 20;

    // immediate offsets
    const I_IMM: u32 = 20;
    const U_IMM: u32 = 12;

    const OP_MASK: u32 = 0b0111_1111;
    const R_MASK: u32 = 0b0001_1111;
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.opcode {
            Opcode::add => write!(f, "{} {}, {}, {}", self.opcode, self.rd, self.rs1, self.rs2),
            Opcode::addi => write!(
                f,
                "{} {}, {}, 0x{:02x}",
                self.opcode, self.rd, self.rs1, self.imm
            ),
            Opcode::auipc | Opcode::lui => {
                write!(f, "{} {}, 0x{:02x}", self.opcode, self.rd, self.imm)
            }
            Opcode::lb => write!(
                f,
                "{} {}, 0x{:02x}({})",
                self.opcode, self.rd, self.imm, self.rs1
            ),
            Opcode::unimp | Opcode::ecall => write!(f, "{}", self.opcode),
        }
    }
}

impl From<Word> for Instruction {
    fn from(word: Word) -> Self {
        let opcode = (word & Instruction::OP_MASK).into();
        match opcode {
            Opcode::add => {
                let rd = ((word >> Instruction::RD) & Instruction::R_MASK).into();
                let rs1 = ((word >> Instruction::RS1) & Instruction::R_MASK).into();
                let rs2 = ((word >> Instruction::RS2) & Instruction::R_MASK).into();

                Instruction {
                    opcode,
                    rd,
                    rs1,
                    rs2,
                    imm: 0,
                }
            }
            Opcode::addi | Opcode::lb => {
                let rd = ((word >> Instruction::RD) & Instruction::R_MASK).into();
                let rs1 = ((word >> Instruction::RS1) & Instruction::R_MASK).into();
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
            Opcode::unimp | Opcode::ecall => Instruction {
                opcode,
                rd: Reg::zero,
                rs1: Reg::zero,
                rs2: Reg::zero,
                imm: 0,
            },
        }
    }
}

impl From<Instruction> for Word {
    fn from(instruction: Instruction) -> Self {
        match instruction.opcode {
            Opcode::add => {
                let opcode: Word = instruction.opcode.into();
                let rd: Word = instruction.rd.into();
                let f3: Word = 0b000;
                let rs1: Word = instruction.rs1.into();
                let rs2: Word = instruction.rs2.into();
                let f7: Word = 0b000;

                opcode
                    | (rd << Instruction::RD)
                    | (f3 << Instruction::F3)
                    | (rs1 << Instruction::RS1)
                    | (rs2 << Instruction::RS2)
                    | (f7 << Instruction::R_F7)
            }
            Opcode::addi | Opcode::lb => {
                let opcode: Word = instruction.opcode.into();
                let rd: Word = instruction.rd.into();
                let f3: Word = 0b000;
                let rs1: Word = instruction.rs1.into();
                let imm: Word = instruction.imm;

                opcode
                    | (rd << Instruction::RD)
                    | (f3 << Instruction::F3)
                    | (rs1 << Instruction::RS1)
                    | (imm << Instruction::I_IMM)
            }
            Opcode::auipc | Opcode::lui => {
                let opcode: Word = instruction.opcode.into();
                let rd: Word = instruction.rd.into();
                let imm: Word = instruction.imm;

                opcode | (rd << Instruction::RD) | (imm << Instruction::U_IMM)
            }
            Opcode::ecall => instruction.opcode.into(),

            Opcode::unimp => 0,
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
        "add" => {
            assert_eq!(instr.operands.len(), 3, "expected 3 operands for add");
            let Operand::Register(rd) = &instr.operands[0] else {
                return Err(anyhow!("expected register"));
            };
            let Operand::Register(rs1) = &instr.operands[1] else {
                return Err(anyhow!("expected register"));
            };
            let Operand::Register(rs2) = &instr.operands[2] else {
                return Err(anyhow!("expected register"));
            };
            vec![Instruction {
                opcode: Opcode::add,
                rd: Reg::try_from(rd.to_string())?,
                rs1: Reg::try_from(rs1.to_string())?,
                rs2: Reg::try_from(rs2.to_string())?,
                imm: 0,
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
        "lb" => {
            assert_eq!(instr.operands.len(), 2, "expected 2 operands for lb");
            let Operand::Register(ref rd) = instr.operands[0] else {
                return Err(anyhow!("expected register"));
            };
            let Operand::OffsetAddress { imm, ref register } = instr.operands[1] else {
                return Err(anyhow!("expected symbol"));
            };
            vec![Instruction {
                opcode: Opcode::lb,
                rd: Reg::try_from(rd.to_string())?,
                rs1: Reg::try_from(register.to_string())?,
                rs2: Reg::zero,
                imm,
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
        _ => todo!("Assemble Instruction: {}", instr.name),
    };

    Ok(instructions)
}

pub type Image = Vec<Word>;

fn assemble_program(program: ast::Program) -> Result<Image> {
    let mut refs: Vec<Ref> = Vec::new();
    let mut symbols = SymbolTable::new();
    let mut data: Vec<Word> = Vec::new();
    let mut instructions: Vec<Instruction> = Vec::new();

    for line in program.lines {
        let address = (size_of::<Word>() * instructions.len()) as Address;
        match line {
            Line::Comment(_) => {}
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
        instructions[r.address as usize / size_of::<Word>()].imm = symbols
            .lookup(&r.name)
            .ok_or(anyhow!("unknown identifier: {:#?}", r.name))?;
    }

    let mut image: Vec<Word> = instructions.into_iter().map(Word::from).collect();
    image.extend_from_slice(&data);

    Ok(image)
}

/// Assembles `input`.
///
/// # Errors
///
/// Returns any errors parsing the input.
pub fn assemble(input: &str) -> Result<Image> {
    let tokens = lexer::tokenize(input);
    let mut parser = Parser::new(tokens);

    let program = parser.parse()?;
    let obj = assemble_program(program)?;

    Ok(obj)
}

/// Builds an executable from `input`.
///
/// # Errors
///
/// Returns any errors reading the input, assembling the program
/// or writing the executable to disk.
pub fn build_exe(input: impl AsRef<Path>, output: impl AsRef<Path>) -> Result<()> {
    const HEADER: &[u8] = b"rme1";

    let source = std::fs::read_to_string(input)?;
    let image = assemble(&source)?;
    let mut bytes = Vec::from(HEADER);
    bytes.extend(image.into_iter().flat_map(Word::to_be_bytes));

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

        build_exe("testdata/hello.s", exe_path.clone()).unwrap();

        // header + text section length + text section + data section length + data section
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

        let mut want: Vec<Word> = vec![
            4195603, 72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100, 33, 10,
        ];

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
                program: "li t0, 64".into(),
                want: Instruction {
                    opcode: Opcode::addi,
                    rd: Reg::t0,
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
            TestCase {
                program: "add t1, t0, a0".into(),
                want: Instruction {
                    opcode: Opcode::add,
                    rd: Reg::t1,
                    rs1: Reg::t0,
                    rs2: Reg::a0,
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
