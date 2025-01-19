use crate::{
    ast::{Directive, Identifier, Instruction, Line, Operand, Program},
    lexer::{Token, TokenType},
};
use anyhow::{anyhow, Result};
use std::{iter::Peekable, vec::IntoIter};

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
            self.instruction(ident)
        } else if self.matches(TokenType::Dot) {
            self.directive()
        } else {
            Err(anyhow!("expected identifier or directive"))
        }
    }

    fn directive(&mut self) -> Result<Line> {
        self.expect(TokenType::Dot)?;
        let ident = self.identifier()?;
        let directive = match ident.as_ref() {
            "globl" => {
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
            _ => todo!(),
        };
        Ok(Line::Directive(directive))
    }

    fn instruction(&mut self, ident: Identifier) -> Result<Line> {
        let instruction = match ident.as_ref() {
            "la" => {
                let rd = self.register()?;
                self.expect(TokenType::Comma)?;
                let symbol = self.identifier()?;
                Instruction {
                    name: ident.to_string(),
                    operands: vec![rd, Operand::Symbol(symbol.0)],
                }
            }
            "li" => {
                let rd = self.register()?;
                self.expect(TokenType::Comma)?;
                let imm = self.immediate()?;
                Instruction {
                    name: ident.to_string(),
                    operands: vec![rd, imm],
                }
            }
            "ecall" => Instruction {
                name: ident.to_string(),
                operands: vec![],
            },
            _ => todo!(),
        };

        Ok(Line::Instruction(instruction))
    }

    fn identifier(&mut self) -> Result<Identifier> {
        let token = self.expect(TokenType::Identifier)?;
        Ok(Identifier(token.lexeme))
    }

    fn register(&mut self) -> Result<Operand> {
        let ident = self.identifier()?;
        Ok(Operand::Register(ident.to_string()))
    }

    fn immediate(&mut self) -> Result<Operand> {
        self.expect(TokenType::Integer)?
            .lexeme
            .parse()
            .map(Operand::Immediate)
            .map_err(|_| anyhow!("expected to parse integer"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer;

    #[test]
    fn parser_parse_returns_program_ast() {
        let program = "
        .globl _start
        .section .text
        _start:
            li a0, 1
            la a1, helloworld
            li a2, 13
            li a7, 64
            ecall
        helloworld:
            .ascii \"Hello, World!\n\"
        ";

        let want = Program {
            lines: vec![
                Line::Directive(Directive::Global("_start".to_string())),
                Line::Directive(Directive::Section(".text".to_string())),
                Line::Label("_start".to_string()),
                Line::Instruction(Instruction {
                    name: "li".to_string(),
                    operands: vec![Operand::Register("a0".to_string()), Operand::Immediate(1)],
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
                    operands: vec![Operand::Register("a2".to_string()), Operand::Immediate(13)],
                }),
                Line::Instruction(Instruction {
                    name: "li".to_string(),
                    operands: vec![Operand::Register("a7".to_string()), Operand::Immediate(64)],
                }),
                Line::Instruction(Instruction {
                    name: "ecall".to_string(),
                    operands: vec![],
                }),
                Line::Label("helloworld".to_string()),
                Line::Directive(Directive::Ascii("\"Hello, World!\n\"".into())),
            ],
        };

        let tokens = lexer::tokenize(program);
        let mut parser = Parser::new(tokens);
        let got = parser.parse().unwrap();
        assert_eq!(want, got);
    }
}
