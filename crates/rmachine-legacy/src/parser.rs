use crate::{
    ast::{Directive, Identifier, Instruction, Line, Operand, Program},
    lexer::{Token, TokenType},
};
use anyhow::{Context, Result, anyhow};
use std::{
    error::Error,
    fmt::{Debug, Display},
    iter::Peekable,
    ops::Neg,
    str::FromStr,
    vec::IntoIter,
};

pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens.into_iter().peekable(),
        }
    }

    pub fn parse(&mut self) -> Result<Program> {
        let mut program = Program { lines: Vec::new() };
        while self.tokens.peek().is_some() {
            let line = self.line()?;
            program.lines.push(line);
        }
        Ok(program)
    }

    fn advance(&mut self) -> Option<Token> {
        self.tokens.next()
    }

    fn peek(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }

    fn matches(&mut self, token_type: TokenType) -> bool {
        self.tokens
            .peek()
            .is_some_and(|t| t.token_type == token_type)
    }

    fn expect(&mut self, token_type: TokenType) -> Result<Token> {
        if self.matches(token_type) {
            self.advance().ok_or(anyhow!("unexpected end of tokens"))
        } else {
            Err(anyhow!("expected token type: {token_type:#?}"))
        }
    }

    fn line(&mut self) -> Result<Line> {
        if self.matches(TokenType::Identifier) {
            let ident = self.identifier()?;
            if self.matches(TokenType::Colon) {
                self.advance();
                return Ok(Line::Label(ident.to_string()));
            }
            self.instruction(ident.to_string())
        } else if self.matches(TokenType::Dot) {
            self.directive()
        } else if self.matches(TokenType::Comment) {
            self.comment()
        } else {
            let curr = self.tokens.peek();
            Err(anyhow!(
                "expected identifier or directive, but got {curr:#?}"
            ))
        }
    }

    fn directive(&mut self) -> Result<Line> {
        self.expect(TokenType::Dot)?;
        let ident = self.identifier()?;
        let directive = match ident.as_ref() {
            "global" => {
                let symbol = self.identifier()?;
                Directive::Global(symbol.to_string())
            }
            "section" => {
                self.expect(TokenType::Dot)?;
                let section = self.identifier()?;
                Directive::Section(format!(".{section}"))
            }
            "ascii" => {
                let string = self.expect(TokenType::String)?;
                Directive::Ascii(string.lexeme)
            }
            _ => todo!("implement directive {}", ident),
        };
        Ok(Line::Directive(directive))
    }

    #[allow(clippy::too_many_lines, clippy::match_same_arms)]
    fn instruction(&mut self, name: String) -> Result<Line> {
        let instruction = match name.as_str() {
            "add" => {
                let rd = self.register()?;
                self.expect(TokenType::Comma)?;
                let rs1 = self.register()?;
                self.expect(TokenType::Comma)?;
                let rs2 = self.register()?;
                Instruction {
                    name,
                    operands: vec![rd, rs1, rs2],
                }
            }
            "addi" => {
                let rd = self.register()?;
                self.expect(TokenType::Comma)?;
                let rs1 = self.register()?;
                self.expect(TokenType::Comma)?;
                let imm = self.immediate_i32()?;
                Instruction {
                    name,
                    operands: vec![rd, rs1, Operand::ImmI32(imm)],
                }
            }
            "auipc" => {
                let rd = self.register()?;
                self.expect(TokenType::Comma)?;
                let imm = self.immediate_u32()?;
                Instruction {
                    name,
                    operands: vec![rd, Operand::ImmU32(imm)],
                }
            }
            "beq" => {
                let rs1 = self.register()?;
                self.expect(TokenType::Comma)?;
                let rs2 = self.register()?;
                self.expect(TokenType::Comma)?;
                let symbol = self.identifier()?;
                Instruction {
                    name,
                    operands: vec![rs1, rs2, Operand::Symbol(symbol.0)],
                }
            }
            "la" => {
                let rd = self.register()?;
                self.expect(TokenType::Comma)?;
                let symbol = self.identifier()?;
                Instruction {
                    name,
                    operands: vec![rd, Operand::Symbol(symbol.0)],
                }
            }
            "lb" => {
                let rd = self.register()?;
                self.expect(TokenType::Comma)?;
                let imm = self.immediate_i32()?;
                self.expect(TokenType::LParen)?;
                let rs = self.identifier()?;
                self.expect(TokenType::RParen)?;
                Instruction {
                    name,
                    operands: vec![
                        rd,
                        Operand::OffsetAddress {
                            imm,
                            register: rs.0,
                        },
                    ],
                }
            }
            "li" => {
                let rd = self.register()?;
                self.expect(TokenType::Comma)?;
                let imm = self.immediate_i32()?;
                Instruction {
                    name,
                    operands: vec![rd, Operand::ImmI32(imm)],
                }
            }
            "lui" => {
                let rd = self.register()?;
                self.expect(TokenType::Comma)?;
                let imm = self.immediate_u32()?;
                Instruction {
                    name,
                    operands: vec![rd, Operand::ImmU32(imm)],
                }
            }
            "ebreak" | "ecall" => Instruction {
                name,
                operands: vec![],
            },
            "j" => {
                let symbol = self.identifier()?;
                Instruction {
                    name,
                    operands: vec![Operand::Symbol(symbol.0)],
                }
            }
            _ => todo!("Implement {name}"),
        };

        Ok(Line::Instruction(instruction))
    }

    fn comment(&mut self) -> Result<Line> {
        let token = self.expect(TokenType::Comment)?;
        Ok(Line::Comment(token.lexeme))
    }

    fn identifier(&mut self) -> Result<Identifier> {
        let token = self.expect(TokenType::Identifier)?;
        Ok(Identifier(token.lexeme))
    }

    fn register(&mut self) -> Result<Operand> {
        let ident = self.identifier()?;
        Ok(Operand::Register(ident.to_string()))
    }

    fn immediate_u32(&mut self) -> Result<u32> {
        self.expect(TokenType::Integer)?
            .lexeme
            .parse()
            .context("failed to parse integer")
    }

    fn immediate_i32(&mut self) -> Result<i32> {
        let mut negative = false;
        if self.expect(TokenType::Minus).is_ok() {
            negative = true;
        }
        let value: i32 = self
            .expect(TokenType::Integer)?
            .lexeme
            .parse()
            .context("failed to parse integer")?;
        Ok(if negative { -value } else { value })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer;
    use pretty_assertions::assert_eq;

    #[test]
    #[allow(clippy::too_many_lines)]
    fn parser_parse_returns_program_ast() {
        let program = "
        .global _start
        .section .text
        _start:
            li a0, 1 # set a0 to 1
            li a0, -1
            lui a1, 42
            auipc a1, 42
            la a1, helloworld
            li a2, 13
            li a7, 64
            lb t0, 0(t0)
            beq t0, zero, end
            addi t0, t0, 1
            add t1, t0, a0
            j loop
            ecall
        end:
            ebreak
        helloworld:
            .ascii \"Hello World!\n\"
        ";

        let want = Program {
            lines: vec![
                Line::Directive(Directive::Global("_start".to_string())),
                Line::Directive(Directive::Section(".text".to_string())),
                Line::Label("_start".to_string()),
                Line::Instruction(Instruction {
                    name: "li".to_string(),
                    operands: vec![Operand::Register("a0".to_string()), Operand::ImmI32(1)],
                }),
                Line::Comment("set a0 to 1".to_string()),
                Line::Instruction(Instruction {
                    name: "li".to_string(),
                    operands: vec![Operand::Register("a0".to_string()), Operand::ImmI32(-1)],
                }),
                Line::Instruction(Instruction {
                    name: "lui".to_string(),
                    operands: vec![Operand::Register("a1".to_string()), Operand::ImmU32(42)],
                }),
                Line::Instruction(Instruction {
                    name: "auipc".to_string(),
                    operands: vec![Operand::Register("a1".to_string()), Operand::ImmU32(42)],
                }),
                Line::Instruction(Instruction {
                    name: "la".to_string(),
                    operands: vec![
                        Operand::Register("a1".to_string()),
                        Operand::Symbol("helloworld".to_string()),
                    ],
                }),
                Line::Instruction(Instruction {
                    name: "li".to_string(),
                    operands: vec![Operand::Register("a2".to_string()), Operand::ImmI32(13)],
                }),
                Line::Instruction(Instruction {
                    name: "li".to_string(),
                    operands: vec![Operand::Register("a7".to_string()), Operand::ImmI32(64)],
                }),
                Line::Instruction(Instruction {
                    name: "lb".to_string(),
                    operands: vec![
                        Operand::Register("t0".to_string()),
                        Operand::OffsetAddress {
                            imm: 0,
                            register: "t0".to_string(),
                        },
                    ],
                }),
                Line::Instruction(Instruction {
                    name: "beq".to_string(),
                    operands: vec![
                        Operand::Register("t0".to_string()),
                        Operand::Register("zero".to_string()),
                        Operand::Symbol("end".to_string()),
                    ],
                }),
                Line::Instruction(Instruction {
                    name: "addi".to_string(),
                    operands: vec![
                        Operand::Register("t0".to_string()),
                        Operand::Register("t0".to_string()),
                        Operand::ImmI32(1),
                    ],
                }),
                Line::Instruction(Instruction {
                    name: "add".to_string(),
                    operands: vec![
                        Operand::Register("t1".to_string()),
                        Operand::Register("t0".to_string()),
                        Operand::Register("a0".to_string()),
                    ],
                }),
                Line::Instruction(Instruction {
                    name: "j".to_string(),
                    operands: vec![Operand::Symbol("loop".to_string())],
                }),
                Line::Instruction(Instruction {
                    name: "ecall".to_string(),
                    operands: vec![],
                }),
                Line::Label("end".to_string()),
                Line::Instruction(Instruction {
                    name: "ebreak".to_string(),
                    operands: vec![],
                }),
                Line::Label("helloworld".to_string()),
                Line::Directive(Directive::Ascii("Hello World!\n".into())),
            ],
        };

        let tokens = lexer::tokenize(program);
        let mut parser = Parser::new(tokens);
        let got = parser.parse().unwrap();
        assert_eq!(want, got);
    }
}
