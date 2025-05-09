#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    Eof,
    Illegal,
    Colon,
    Comma,
    Dot,
    LParen,
    RParen,
    Integer,
    Identifier,
    String,
    Comment,
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
    char: Option<char>,
}

impl Lexer {
    fn new(input: &str) -> Self {
        let input: Vec<char> = input.chars().collect();
        Self {
            input,
            pos: 0,
            next_pos: 0,
            char: None,
        }
    }

    fn peek_char(&self) -> Option<char> {
        if self.next_pos >= self.input.len() {
            None
        } else {
            Some(self.input[self.next_pos])
        }
    }

    fn read_char(&mut self) {
        self.char = self.peek_char();
        self.pos = self.next_pos;
        self.next_pos += 1;
    }

    fn skip_whitespace(&mut self) {
        while self.char.is_some_and(|c| c.is_ascii_whitespace()) {
            self.read_char();
        }
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        if let Some(ch) = self.char {
            match ch {
                '#' => {
                    self.read_char();
                    self.skip_whitespace();
                    let start = self.pos;
                    while self.char.is_some_and(|c| c != '\n') {
                        self.read_char();
                    }
                    let comment: String = self.input[start..self.pos].iter().collect();
                    Token::new(TokenType::Comment, comment)
                }
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
                '(' => {
                    self.read_char();
                    Token::new(TokenType::LParen, "(")
                }
                ')' => {
                    self.read_char();
                    Token::new(TokenType::RParen, ")")
                }
                'a'..='z' | '_' => {
                    let start = self.pos;
                    while self
                        .char
                        .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
                    {
                        self.read_char();
                    }
                    let ident: String = self.input[start..self.pos].iter().collect();
                    Token::new(TokenType::Identifier, ident)
                }
                '0'..='9' => {
                    let start = self.pos;
                    while self.char.is_some_and(|c| c.is_ascii_digit()) {
                        self.read_char();
                    }
                    let lexeme = self.input[start..self.pos].iter().collect::<String>();
                    Token::new(TokenType::Integer, lexeme)
                }
                '"' => {
                    self.read_char(); // consume the opening quote
                    let start = self.pos;
                    while self.char.is_some_and(|c| c != '"') {
                        self.read_char();
                    }
                    let lexeme = self.input[start..self.pos].iter().collect::<String>();
                    self.read_char(); // consume the closing quote
                    Token::new(TokenType::String, lexeme)
                }
                _ => Token::new(TokenType::Illegal, ch),
            }
        } else {
            Token::new(TokenType::Eof, "")
        }
    }
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];
    let mut lexer = Lexer::new(input);
    lexer.read_char();

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
                program: "lb t0 0(t0)".to_string(),
                want: vec![
                    Token {
                        token_type: TokenType::Identifier,
                        lexeme: "lb".to_string(),
                    },
                    Token {
                        token_type: TokenType::Identifier,
                        lexeme: "t0".to_string(),
                    },
                    Token {
                        token_type: TokenType::Integer,
                        lexeme: "0".to_string(),
                    },
                    Token {
                        token_type: TokenType::LParen,
                        lexeme: "(".to_string(),
                    },
                    Token {
                        token_type: TokenType::Identifier,
                        lexeme: "t0".to_string(),
                    },
                    Token {
                        token_type: TokenType::RParen,
                        lexeme: ")".to_string(),
                    },
                ],
            },
            TestCase {
                program: "# this is a comment\nli a0, 42".to_string(),
                want: vec![
                    Token {
                        token_type: TokenType::Comment,
                        lexeme: "this is a comment".to_string(),
                    },
                    Token {
                        token_type: TokenType::Identifier,
                        lexeme: "li".to_string(),
                    },
                    Token {
                        token_type: TokenType::Identifier,
                        lexeme: "a0".to_string(),
                    },
                    Token {
                        token_type: TokenType::Comma,
                        lexeme: ",".to_string(),
                    },
                    Token {
                        token_type: TokenType::Integer,
                        lexeme: "42".to_string(),
                    },
                ],
            },
            TestCase {
                program: "li a0, 42 # this is a comment".to_string(),
                want: vec![
                    Token {
                        token_type: TokenType::Identifier,
                        lexeme: "li".to_string(),
                    },
                    Token {
                        token_type: TokenType::Identifier,
                        lexeme: "a0".to_string(),
                    },
                    Token {
                        token_type: TokenType::Comma,
                        lexeme: ",".to_string(),
                    },
                    Token {
                        token_type: TokenType::Integer,
                        lexeme: "42".to_string(),
                    },
                    Token {
                        token_type: TokenType::Comment,
                        lexeme: "this is a comment".to_string(),
                    },
                ],
            },
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
                        lexeme: "Hello World!\n".into(),
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
