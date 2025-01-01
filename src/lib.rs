#![allow(
    unused,
    clippy::cast_possible_truncation,
    clippy::needless_pass_by_value
)]
use std::collections::HashMap;
use std::{iter, str::Chars};

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
    out: Vec<Word>,
}

impl Machine {
    const SYSCALL_WRITE: u32 = 64;
    const FD_STDOUT: u32 = 1;

    fn new() -> Self {
        Self::default()
    }

    fn load_image(&mut self, image: Vec<Word>) {
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

    fn run(&mut self) -> Result<()> {
        loop {
            let instruction = self.next()?;
            self.pc += 1;

            println!("pc = {:#?}", self.pc);
            println!("regs = {:#?}", self.regs);
            println!("instr = {instruction:#?}");

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
                Opcode::EnvironmentCall => {
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

                                    _ => panic!("Unknown fd: {fd}"),
                                }
                            }
                        }
                        _ => todo!(),
                    }
                }
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
    a6,
    a7,
}

impl TryFrom<Word> for Reg {
    type Error = Error;

    fn try_from(word: Word) -> Result<Self> {
        match word {
            0b00000 => Ok(Reg::zero),
            0b01010 => Ok(Reg::a0),
            0b01011 => Ok(Reg::a1),
            0b01100 => Ok(Reg::a2),
            0b10001 => Ok(Reg::a7),
            _ => Err(Error::RegisterUnknown(word)),
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

#[derive(Debug, PartialEq, Default)]
struct Instruction {
    opcode: Opcode,
    rd: Reg,
    rs1: Reg,
    rs2: Reg,
    imm: u32,
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

impl TryFrom<Word> for Instruction {
    type Error = Error;

    fn try_from(word: Word) -> Result<Self> {
        let opcode = (word & Instruction::OP_MASK).try_into()?;
        match opcode {
            Opcode::AddImmediate => {
                let rd = ((word >> Instruction::RD) & Instruction::R_MASK).try_into()?;
                let rs1 = ((word >> Instruction::I_RS1) & Instruction::R_MASK).try_into()?;
                let imm = (word >> Instruction::I_IMM);
                Ok(Instruction {
                    opcode,
                    rd,
                    rs1,
                    imm,
                    ..Default::default()
                })
            }
            Opcode::AddUpperImmediateToProgramCounter => {
                let rd = ((word >> Instruction::RD) & Instruction::R_MASK).try_into()?;
                let imm = (word >> Instruction::U_IMM);
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
                let rd = ((word >> Instruction::RD) & Instruction::R_MASK).try_into()?;
                let imm = (word >> Instruction::U_IMM);
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

                opcode
                    | (rd << Instruction::RD)
                    | (f3 << Instruction::I_F3)
                    | (rs1 << Instruction::I_RS1)
                    | (imm << Instruction::I_IMM)
            }
            Opcode::AddUpperImmediateToProgramCounter | Opcode::LoadUpperImmediate => {
                let opcode: Word = instruction.opcode.into();
                let rd: Word = instruction.rd.into();
                let imm: Word = instruction.imm;

                opcode | (rd << Instruction::RD) | (imm << Instruction::U_IMM)
            }
            Opcode::EnvironmentCall => instruction.opcode.into(),
        }
    }
}

fn assemble(input: &str) -> Vec<Word> {
    let tokens = tokenize(input);
    let instructions = parse(tokens);
    instructions.into_iter().map(Word::from).collect()
}

fn parse(tokens: Vec<Token>) -> Vec<Instruction> {
    let mut instructions = Vec::new();
    let mut token_iter = tokens.into_iter().peekable();

    while let Some(Token::Opcode(opcode)) = token_iter.next() {
        match opcode.as_str() {
            "li" => {
                let Some(reg) = token_iter.next() else {
                    panic!("missing destination register")
                };
                let Token::Register(rd) = reg else {
                    panic!("expected register, got: {reg:?}")
                };
                let Some(Token::Comma) = token_iter.next() else {
                    panic!("missing comma")
                };
                let Some(Token::Integer(imm)) = token_iter.next() else {
                    panic!("missing immediate value")
                };
                instructions.push(Instruction {
                    opcode: Opcode::AddImmediate,
                    rd,
                    imm,
                    ..Default::default()
                });
            }
            _ => panic!("unrecognized opcode: {opcode:#?}"),
        }
    }

    instructions
}

#[derive(Debug, PartialEq)]
enum Token {
    Comma,
    Opcode(String),
    Register(Reg),
    Integer(u32),
    Identifier(String),
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            ch if ch.is_whitespace() => continue,
            ',' => tokens.push(Token::Comma),
            '0'..='9' => {
                let n: u32 = iter::once(ch)
                    .chain(iter::from_fn(|| {
                        chars.by_ref().next_if(char::is_ascii_digit)
                    }))
                    .collect::<String>()
                    .parse()
                    .expect("couldn't parse number");

                tokens.push(Token::Integer(n));
            }
            'a'..='z' => {
                let ident: String = iter::once(ch)
                    .chain(iter::from_fn(|| {
                        chars.by_ref().next_if(char::is_ascii_alphanumeric)
                    }))
                    .collect();
                tokens.push(lookup_ident(ident));
            }
            _ => panic!("unrecognized char: {ch:#?}"),
        }
    }

    tokens
}

fn lookup_ident(ident: String) -> Token {
    match ident.as_str() {
        "a0" => Token::Register(Reg::a0),
        "a1" => Token::Register(Reg::a1),
        "a2" => Token::Register(Reg::a2),
        "a7" => Token::Register(Reg::a7),
        "li" => Token::Opcode(ident),
        _ => Token::Identifier(ident),
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
                word: 0b0000_0010_0000_0101_1000_0101_1001_0011,
                instruction: Instruction {
                    opcode: Opcode::AddImmediate,
                    rd: Reg::a1,
                    rs1: Reg::a1,
                    imm: 32,
                    ..Default::default()
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
                opcode: Opcode::AddImmediate,
                rd: Reg::a0,
                imm: 1,
                ..Default::default()
            },
            Instruction {
                opcode: Opcode::AddUpperImmediateToProgramCounter,
                rd: Reg::a1,
                imm: 0,
                ..Default::default()
            },
            Instruction {
                opcode: Opcode::AddImmediate,
                rd: Reg::a1,
                rs1: Reg::a1,
                imm: 30,
                ..Default::default()
            },
            Instruction {
                opcode: Opcode::AddImmediate,
                rd: Reg::a2,
                imm: 13,
                ..Default::default()
            },
            Instruction {
                opcode: Opcode::AddImmediate,
                rd: Reg::a7,
                imm: 64,
                ..Default::default()
            },
            Instruction {
                opcode: Opcode::EnvironmentCall,
                ..Default::default()
            },
        ];
        for (i, instruction) in instructons.into_iter().enumerate() {
            machine.mem.set(i as Address, instruction.into());
        }
        let hello_world: Vec<Word> = "Hello World!\n".chars().map(|c| c as Word).collect();
        for (i, c) in hello_world.iter().enumerate() {
            machine.mem.set((i + 32) as Address, *c);
        }

        assert_err_eq!(machine.run(), Error::OpcodeUnknown(0));

        let got = machine.out;
        assert_eq!(got, hello_world);
    }

    #[test]
    fn add_immediate_1() {
        let image = assemble("li a0, 1");

        let mut machine = Machine::default();
        machine.load_image(image);

        assert_err_eq!(machine.run(), Error::OpcodeUnknown(0));

        let want = 1;
        let got = machine.regs.get(Reg::a0);
        assert_eq!(want, got, "wrong a0: {want}, expected: {got}");
    }

    #[test]
    fn test_assemble() {
        let program = "li a0, 1";

        let instruction = Instruction {
            opcode: Opcode::AddImmediate,
            rd: Reg::a0,
            imm: 1,
            ..Default::default()
        };
        let want: Vec<Word> = vec![instruction.into()];

        let got = assemble(program);
        assert_eq!(want, got);
    }

    #[test]
    fn tokenize_returns_tokens() {
        let want = vec![
            Token::Opcode("li".into()),
            Token::Register(Reg::a0),
            Token::Comma,
            Token::Integer(1),
        ];

        let got = tokenize("li a0, 1");

        assert_eq!(want, got);
    }

    #[test]
    fn parse_returns_instructions() {
        struct TestCase {
            program: String,
            want: Vec<Instruction>,
        }

        let cases = vec![
            TestCase {
                program: "li a0, 1".into(),
                want: vec![Instruction {
                    opcode: Opcode::AddImmediate,
                    rd: Reg::a0,
                    imm: 1,
                    ..Default::default()
                }],
            },
            TestCase {
                program: "li a1, 32".into(),
                want: vec![Instruction {
                    opcode: Opcode::AddImmediate,
                    rd: Reg::a1,
                    imm: 32,
                    ..Default::default()
                }],
            },
            TestCase {
                program: "li a2, 13".into(),
                want: vec![Instruction {
                    opcode: Opcode::AddImmediate,
                    rd: Reg::a2,
                    imm: 13,
                    ..Default::default()
                }],
            },
            TestCase {
                program: "li a7, 64".into(),
                want: vec![Instruction {
                    opcode: Opcode::AddImmediate,
                    rd: Reg::a7,
                    imm: 64,
                    ..Default::default()
                }],
            },
        ];

        for case in cases {
            let tokens = tokenize(&case.program);
            let got = parse(tokens);
            assert_eq!(case.want, got);
        }
    }
}
