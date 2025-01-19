#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    Eof,
    Illegal,
    Colon,
    Comma,
    Dot,
    Integer,
    Identifier,
    String,
}

#[derive(Debug, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
}

impl Token {
    fn new(token_type: TokenType, lexeme: impl Into<String>) -> Self {
        Self {
            token_type,
            lexeme: lexeme.into(),
        }
    }
}

struct Lexer {
    input: Vec<char>,
    pos: usize,
    next_pos: usize,
    char: char,
}

impl Lexer {
    fn new(input: &str) -> Self {
        let input: Vec<char> = input.chars().collect();
        let char = input[0];
        Self {
            input,
            pos: 0,
            next_pos: 1,
            char,
        }
    }

    fn peek_char(&self) -> char {
        if self.next_pos >= self.input.len() {
            '\0'
        } else {
            self.input[self.next_pos]
        }
    }

    fn read_char(&mut self) {
        self.char = self.peek_char();
        self.pos = self.next_pos;
        self.next_pos += 1;
    }

    fn skip_whitespace(&mut self) {
        while self.char.is_ascii_whitespace() {
            self.read_char();
        }
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        match self.char {
            ':' => {
                self.read_char();
                Token::new(TokenType::Colon, ":")
            }
            ',' => {
                self.read_char();
                Token::new(TokenType::Comma, ",")
            }
            '.' => {
                self.read_char();
                Token::new(TokenType::Dot, ".")
            }
            'a'..='z' | '_' => {
                let start = self.pos;
                while self.char.is_ascii_alphanumeric() || self.char == '_' {
                    self.read_char();
                }
                let ident: String = self.input[start..self.pos].iter().collect();
                Token::new(TokenType::Identifier, ident)
            }
            '0'..='9' => {
                let start = self.pos;
                while self.char.is_ascii_digit() {
                    self.read_char();
                }
                let lexeme = self.input[start..self.pos].iter().collect::<String>();
                Token::new(TokenType::Integer, lexeme)
            }
            '"' => {
                let start = self.pos;
                self.read_char(); // consume the opening quote
                while self.char != '"' {
                    self.read_char();
                }
                self.read_char(); // consume the closing quote
                let lexeme = self.input[start..self.pos].iter().collect::<String>();
                Token::new(TokenType::String, lexeme)
            }
            '\0' => Token::new(TokenType::Eof, ""),
            _ => Token::new(TokenType::Illegal, self.char),
        }
    }
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];
    let mut lexer = Lexer::new(input);

    loop {
        let token = lexer.next_token();
        match token.token_type {
            TokenType::Eof => break,
            TokenType::Illegal => {
                panic!("illegal token: {token:#?}");
            }
            _ => tokens.push(token),
        }
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::too_many_lines)]
    fn tokenize_returns_tokens() {
        struct TestCase {
            program: String,
            want: Vec<Token>,
        }
        let cases = [
            TestCase {
                program: ".ascii \"Hello World!\n\"".to_string(),
                want: vec![
                    Token {
                        token_type: TokenType::Dot,
                        lexeme: ".".to_string(),
                    },
                    Token {
                        token_type: TokenType::Identifier,
                        lexeme: "ascii".into(),
                    },
                    Token {
                        token_type: TokenType::String,
                        lexeme: "\"Hello World!\n\"".into(),
                    },
                ],
            },
            TestCase {
                program: ".global _start".to_string(),
                want: vec![
                    Token {
                        token_type: TokenType::Dot,
                        lexeme: ".".to_string(),
                    },
                    Token {
                        token_type: TokenType::Identifier,
                        lexeme: "global".into(),
                    },
                    Token {
                        token_type: TokenType::Identifier,
                        lexeme: "_start".into(),
                    },
                ],
            },
            TestCase {
                program: ".section .text".to_string(),
                want: vec![
                    Token {
                        token_type: TokenType::Dot,
                        lexeme: ".".to_string(),
                    },
                    Token {
                        token_type: TokenType::Identifier,
                        lexeme: "section".into(),
                    },
                    Token {
                        token_type: TokenType::Dot,
                        lexeme: ".".to_string(),
                    },
                    Token {
                        token_type: TokenType::Identifier,
                        lexeme: "text".into(),
                    },
                ],
            },
            TestCase {
                program: "_start:".to_string(),
                want: vec![
                    Token {
                        token_type: TokenType::Identifier,
                        lexeme: "_start".into(),
                    },
                    Token {
                        token_type: TokenType::Colon,
                        lexeme: ":".into(),
                    },
                ],
            },
            TestCase {
                program: "li a0, 1".into(),
                want: vec![
                    Token {
                        token_type: TokenType::Identifier,
                        lexeme: "li".into(),
                    },
                    Token {
                        token_type: TokenType::Identifier,
                        lexeme: "a0".into(),
                    },
                    Token {
                        token_type: TokenType::Comma,
                        lexeme: ",".into(),
                    },
                    Token {
                        token_type: TokenType::Integer,
                        lexeme: "1".into(),
                    },
                ],
            },
            TestCase {
                program: "li a1, 32".into(),
                want: vec![
                    Token {
                        token_type: TokenType::Identifier,
                        lexeme: "li".into(),
                    },
                    Token {
                        token_type: TokenType::Identifier,
                        lexeme: "a1".into(),
                    },
                    Token {
                        token_type: TokenType::Comma,
                        lexeme: ",".into(),
                    },
                    Token {
                        token_type: TokenType::Integer,
                        lexeme: "32".into(),
                    },
                ],
            },
            TestCase {
                program: "li a2, 13".into(),
                want: vec![
                    Token {
                        token_type: TokenType::Identifier,
                        lexeme: "li".into(),
                    },
                    Token {
                        token_type: TokenType::Identifier,
                        lexeme: "a2".into(),
                    },
                    Token {
                        token_type: TokenType::Comma,
                        lexeme: ",".into(),
                    },
                    Token {
                        token_type: TokenType::Integer,
                        lexeme: "13".into(),
                    },
                ],
            },
            TestCase {
                program: "li a7, 64".into(),
                want: vec![
                    Token {
                        token_type: TokenType::Identifier,
                        lexeme: "li".into(),
                    },
                    Token {
                        token_type: TokenType::Identifier,
                        lexeme: "a7".into(),
                    },
                    Token {
                        token_type: TokenType::Comma,
                        lexeme: ",".into(),
                    },
                    Token {
                        token_type: TokenType::Integer,
                        lexeme: "64".into(),
                    },
                ],
            },
            TestCase {
                program: "la a1, helloworld".into(),
                want: vec![
                    Token {
                        token_type: TokenType::Identifier,
                        lexeme: "la".into(),
                    },
                    Token {
                        token_type: TokenType::Identifier,
                        lexeme: "a1".into(),
                    },
                    Token {
                        token_type: TokenType::Comma,
                        lexeme: ",".into(),
                    },
                    Token {
                        token_type: TokenType::Identifier,
                        lexeme: "helloworld".into(),
                    },
                ],
            },
            TestCase {
                program: "ecall".into(),
                want: vec![Token {
                    token_type: TokenType::Identifier,
                    lexeme: "ecall".into(),
                }],
            },
        ];

        for case in cases {
            let got = tokenize(&case.program);
            assert_eq!(case.want, got);
        }
    }
}
