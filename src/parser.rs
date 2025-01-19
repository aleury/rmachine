use std::{iter::Peekable, vec::IntoIter};

use anyhow::{anyhow, Result};

use crate::{
    ast::{Identifier, Line, Operand, Program},
    lexer::{Token, TokenType},
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
            self.advance().ok_or(anyhow!("expected token"))
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
            return self.instruction(ident);
        }
        todo!("implement directive parsing")
    }

    fn instruction(&mut self, ident: Identifier) -> Result<Line> {
        let instruction = match ident.0.as_str() {
            "li" => {
                let rd = self.register()?;
                self.expect(TokenType::Comma)?;
                let imm = self.immediate()?;
                Line::Instruction {
                    name: ident,
                    operands: vec![rd, imm],
                }
            }
            "ecall" => Line::Instruction {
                name: ident,
                operands: vec![],
            },
            _ => todo!(),
        };

        Ok(instruction)
    }

    fn identifier(&mut self) -> Result<Identifier> {
        let token = self.expect(TokenType::Identifier)?;
        Ok(Identifier(token.lexeme.clone()))
    }

    fn register(&mut self) -> Result<Operand> {
        let token = self.expect(TokenType::Identifier)?;
        Ok(Operand::Register(token.lexeme.clone()))
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
        _start:
            li a0, 1
            li a1, 32
            li a2, 13
            li a7, 64
            ecall
        ";

        let want = Program {
            lines: vec![
                Line::Label(Identifier("_start".into())),
                Line::Instruction {
                    name: Identifier("li".into()),
                    operands: vec![Operand::Register("a0".into()), Operand::Immediate(1)],
                },
                Line::Instruction {
                    name: Identifier("li".into()),
                    operands: vec![Operand::Register("a1".into()), Operand::Immediate(32)],
                },
                Line::Instruction {
                    name: Identifier("li".into()),
                    operands: vec![Operand::Register("a2".into()), Operand::Immediate(13)],
                },
                Line::Instruction {
                    name: Identifier("li".into()),
                    operands: vec![Operand::Register("a7".into()), Operand::Immediate(64)],
                },
                Line::Instruction {
                    name: Identifier("ecall".into()),
                    operands: vec![],
                },
            ],
        };

        let tokens = lexer::tokenize(program);
        let mut parser = Parser::new(tokens);
        let got = parser.parse().unwrap();
        assert_eq!(want, got);
    }
}
