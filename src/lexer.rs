#[derive(Debug, PartialEq)]
pub enum Token {
    Eof,
    Illegal(String),
    Colon,
    Comma,
    Dot,
    Integer(u32),
    Identifier(String),
    String(String),
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
                Token::Colon
            }
            ',' => {
                self.read_char();
                Token::Comma
            }
            '.' => {
                self.read_char();
                Token::Dot
            }
            'a'..='z' | '_' => {
                let start = self.pos;
                while self.char.is_ascii_alphanumeric() || self.char == '_' {
                    self.read_char();
                }
                let ident: String = self.input[start..self.pos].iter().collect();
                Token::Identifier(ident)
            }
            '0'..='9' => {
                let start = self.pos;
                while self.char.is_ascii_digit() {
                    self.read_char();
                }
                let lexeme = self.input[start..self.pos].iter().collect::<String>();
                if let Ok(int) = lexeme.parse::<u32>() {
                    Token::Integer(int)
                } else {
                    Token::Illegal(lexeme)
                }
            }
            '"' => {
                println!("{:#?}", self.char);
                let start = self.pos;
                self.read_char(); // consume the opening quote
                while self.char != '"' {
                    println!("{:#?}", self.char);
                    self.read_char();
                }
                println!("{:#?}", self.char);
                self.read_char(); // consume the closing quote
                let stuff: Vec<char> = self.input[start..self.pos].iter().cloned().collect();
                println!("{:#?}", stuff);
                let lexeme = self.input[start..self.pos].iter().collect::<String>();
                Token::String(lexeme)
            }
            '\0' => Token::Eof,
            _ => Token::Illegal(self.char.to_string()),
        }
    }
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];
    let mut lexer = Lexer::new(input);

    loop {
        let token = lexer.next_token();
        match token {
            Token::Eof => break,
            Token::Illegal(char) => {
                panic!("illegal token: {char:#?}");
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
    fn tokenize_returns_tokens() {
        struct TestCase {
            program: String,
            want: Vec<Token>,
        }
        let cases = [
            TestCase {
                program: ".ascii \"Hello World!\n\"".to_string(),
                want: vec![
                    Token::Dot,
                    Token::Identifier("ascii".into()),
                    Token::String("\"Hello World!\n\"".into()),
                ],
            },
            TestCase {
                program: ".global _start".to_string(),
                want: vec![
                    Token::Dot,
                    Token::Identifier("global".into()),
                    Token::Identifier("_start".into()),
                ],
            },
            TestCase {
                program: ".section .text".to_string(),
                want: vec![
                    Token::Dot,
                    Token::Identifier("section".into()),
                    Token::Dot,
                    Token::Identifier("text".into()),
                ],
            },
            TestCase {
                program: "_start:".to_string(),
                want: vec![Token::Identifier("_start".into()), Token::Colon],
            },
            TestCase {
                program: "li a0, 1".into(),
                want: vec![
                    Token::Identifier("li".into()),
                    Token::Identifier("a0".into()),
                    Token::Comma,
                    Token::Integer(1),
                ],
            },
            TestCase {
                program: "li a1, 32".into(),
                want: vec![
                    Token::Identifier("li".into()),
                    Token::Identifier("a1".into()),
                    Token::Comma,
                    Token::Integer(32),
                ],
            },
            TestCase {
                program: "li a2, 13".into(),
                want: vec![
                    Token::Identifier("li".into()),
                    Token::Identifier("a2".into()),
                    Token::Comma,
                    Token::Integer(13),
                ],
            },
            TestCase {
                program: "li a7, 64".into(),
                want: vec![
                    Token::Identifier("li".into()),
                    Token::Identifier("a7".into()),
                    Token::Comma,
                    Token::Integer(64),
                ],
            },
            TestCase {
                program: "la a1, helloworld".into(),
                want: vec![
                    Token::Identifier("la".into()),
                    Token::Identifier("a1".into()),
                    Token::Comma,
                    Token::Identifier("helloworld".into()),
                ],
            },
            TestCase {
                program: "ecall".into(),
                want: vec![Token::Identifier("ecall".into())],
            },
        ];

        for case in cases {
            let got = tokenize(&case.program);
            assert_eq!(case.want, got);
        }
    }
}
