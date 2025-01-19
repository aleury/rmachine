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
                return Ok(Line::Label(ident));
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
        let directive = match ident.0.as_str() {
            "globl" => {
                let symbol = self.identifier()?;
                Line::Directive(Directive::Global(symbol))
            }
            "section" => {
                self.expect(TokenType::Dot)?;
                let section = self.identifier()?;
                Line::Directive(Directive::Section(section))
            }
            "ascii" => {
                let string = self.expect(TokenType::String)?;
                Line::Directive(Directive::Ascii(string.lexeme))
            }
            _ => todo!(),
        };
        Ok(directive)
    }

    fn instruction(&mut self, ident: Identifier) -> Result<Line> {
        let instruction = match ident.0.as_str() {
            "la" => {
                let rd = self.register()?;
                self.expect(TokenType::Comma)?;
                let symbol = self.identifier()?;
                Instruction {
                    name: ident,
                    operands: vec![rd, Operand::Symbol(symbol)],
                }
            }
            "li" => {
                let rd = self.register()?;
                self.expect(TokenType::Comma)?;
                let imm = self.immediate()?;
                Instruction {
                    name: ident,
                    operands: vec![rd, imm],
                }
            }
            "ecall" => Instruction {
                name: ident,
                operands: vec![],
            },
            _ => todo!(),
        };

        Ok(Line::Instruction(instruction))
    }

    fn identifier(&mut self) -> Result<Identifier> {
        let token = self.expect(TokenType::Identifier)?;
        Ok(Identifier(token.lexeme.clone()))
    }

    fn register(&mut self) -> Result<Operand> {
        let ident = self.identifier()?;
        Ok(Operand::Register(ident))
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
                Line::Directive(Directive::Global(Identifier("_start".into()))),
                Line::Directive(Directive::Section(Identifier("text".into()))),
                Line::Label(Identifier("_start".into())),
                Line::Instruction(Instruction {
                    name: Identifier("li".to_string()),
                    operands: vec![
                        Operand::Register(Identifier("a0".to_string())),
                        Operand::Immediate(1),
                    ],
                }),
                Line::Instruction(Instruction {
                    name: Identifier("la".into()),
                    operands: vec![
                        Operand::Register(Identifier("a1".into())),
                        Operand::Symbol(Identifier("helloworld".into())),
                    ],
                }),
                Line::Instruction(Instruction {
                    name: Identifier("li".into()),
                    operands: vec![
                        Operand::Register(Identifier("a2".into())),
                        Operand::Immediate(13),
                    ],
                }),
                Line::Instruction(Instruction {
                    name: Identifier("li".into()),
                    operands: vec![
                        Operand::Register(Identifier("a7".into())),
                        Operand::Immediate(64),
                    ],
                }),
                Line::Instruction(Instruction {
                    name: Identifier("ecall".into()),
                    operands: vec![],
                }),
                Line::Label(Identifier("helloworld".to_string())),
                Line::Directive(Directive::Ascii("\"Hello, World!\n\"".into())),
            ],
        };

        let tokens = lexer::tokenize(program);
        let mut parser = Parser::new(tokens);
        let got = parser.parse().unwrap();
        assert_eq!(want, got);
    }
}
