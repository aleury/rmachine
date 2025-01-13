use anyhow::anyhow;

use crate::lexer::Token;

#[allow(non_camel_case_types)]
#[derive(Debug, Eq, PartialEq, Hash, PartialOrd)]
pub enum InstructionName {
    addi,
    auipc,
    ecall,
    la,
    li,
    lui,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Eq, PartialEq, Hash, PartialOrd)]
pub enum RegisterName {
    zero,
    a0,
    a1,
    a2,
    a7,
}

impl TryFrom<String> for RegisterName {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "zero" => Ok(RegisterName::zero),
            "a0" => Ok(RegisterName::a0),
            "a1" => Ok(RegisterName::a1),
            "a2" => Ok(RegisterName::a2),
            "a7" => Ok(RegisterName::a7),
            _ => Err(anyhow!("unknown register: {value:#?}")),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct InstructionStatement {
    pub name: InstructionName,
    pub rd: RegisterName,
    pub rs1: RegisterName,
    pub rs2: RegisterName,
    pub imm: u32,
}

impl TryFrom<String> for InstructionName {
    type Error = anyhow::Error;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        match value.as_str() {
            "addi" => Ok(InstructionName::addi),
            "auipc" => Ok(InstructionName::auipc),
            "ecall" => Ok(InstructionName::ecall),
            "li" => Ok(InstructionName::li),
            "la" => Ok(InstructionName::la),
            "lui" => Ok(InstructionName::lui),
            _ => Err(anyhow!("unknown instruction: {value:#?}")),
        }
    }
}

pub fn parse(tokens: Vec<Token>) -> anyhow::Result<Vec<InstructionStatement>> {
    let mut instructions = Vec::new();
    let mut token_iter = tokens.into_iter().peekable();

    while let Some(Token::Identifier(ident)) = token_iter.next() {
        let instr_name = ident.try_into()?;
        let instruction = match instr_name {
            InstructionName::li => {
                let Some(Token::Identifier(rd)) = token_iter.next() else {
                    return Err(anyhow!("expected destination register: rd"));
                };
                let Some(Token::Comma) = token_iter.next() else {
                    return Err(anyhow!("syntax error: missing comma"));
                };
                let Some(Token::Integer(imm)) = token_iter.next() else {
                    return Err(anyhow!("syntax error: missing immediate value operand"));
                };
                InstructionStatement {
                    name: instr_name,
                    rd: rd.try_into()?,
                    imm,
                    rs1: RegisterName::zero,
                    rs2: RegisterName::zero,
                }
            }
            InstructionName::ecall => InstructionStatement {
                name: instr_name,
                rd: RegisterName::zero,
                rs1: RegisterName::zero,
                rs2: RegisterName::zero,
                imm: 0,
            },
            _ => todo!(),
        };
        instructions.push(instruction);
    }

    Ok(instructions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer;

    #[test]
    fn parse_returns_instructions() {
        struct TestCase {
            program: String,
            want: Vec<InstructionStatement>,
        }

        let cases = vec![
            TestCase {
                program: "li a0, 1".into(),
                want: vec![InstructionStatement {
                    name: InstructionName::li,
                    rd: RegisterName::a0,
                    rs1: RegisterName::zero,
                    rs2: RegisterName::zero,
                    imm: 1,
                }],
            },
            TestCase {
                program: "li a1, 32".into(),
                want: vec![InstructionStatement {
                    name: InstructionName::li,
                    rd: RegisterName::a1,
                    rs1: RegisterName::zero,
                    rs2: RegisterName::zero,
                    imm: 32,
                }],
            },
            TestCase {
                program: "li a2, 13".into(),
                want: vec![InstructionStatement {
                    name: InstructionName::li,
                    rd: RegisterName::a2,
                    rs1: RegisterName::zero,
                    rs2: RegisterName::zero,
                    imm: 13,
                }],
            },
            TestCase {
                program: "li a7, 64".into(),
                want: vec![InstructionStatement {
                    name: InstructionName::li,
                    rd: RegisterName::a7,
                    rs1: RegisterName::zero,
                    rs2: RegisterName::zero,
                    imm: 64,
                }],
            },
            TestCase {
                program: "ecall".into(),
                want: vec![InstructionStatement {
                    name: InstructionName::ecall,
                    rd: RegisterName::zero,
                    rs1: RegisterName::zero,
                    rs2: RegisterName::zero,
                    imm: 0,
                }],
            },
        ];

        for case in cases {
            let tokens = lexer::tokenize(&case.program);
            let got = parse(tokens).unwrap();
            assert_eq!(case.want, got);
        }
    }
}
