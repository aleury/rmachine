#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    Eof,
    Illegal,
    Colon,
    Comma,
    Dot,
    Minus,
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
                '-' => {
                    self.read_char();
                    Token::new(TokenType::Minus, "-")
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
    use test_case::test_case;

    #[test_case(
        "li a0, -1",
        vec![
            token_ident("li"),
            token_ident("a0"),
            token_comma(),
            token_minus(),
            token_int("1"),
        ] ;
        "load immediate with negative"
    )]
    #[test_case(
        "lb t0, 0(t0)",
        vec![
            token_ident("lb"),
            token_ident("t0"),
            token_comma(),
            token_int("0"),
            token_lparen(),
            token_ident("t0"),
            token_rparen(),
        ] ;
        "load byte with offset addressing"
    )]
    #[test_case(
        "# this is a comment\nli a0, 42",
        vec![
            token_comment("this is a comment"),
            token_ident("li"),
            token_ident("a0"),
            token_comma(),
            token_int("42"),
        ] ;
        "comment on separate line"
    )]
    #[test_case(
        "li a0, 42 # this is a comment",
        vec![
            token_ident("li"),
            token_ident("a0"),
            token_comma(),
            token_int("42"),
            token_comment("this is a comment"),
        ] ;
        "inline comment"
    )]
    #[test_case(
        ".ascii \"Hello World!\n\"",
        vec![
            token_dot(),
            token_ident("ascii"),
            token_string("Hello World!\n"),
        ] ;
        "ascii directive with string"
    )]
    #[test_case(
        ".global _start",
        vec![
            token_dot(),
            token_ident("global"),
            token_ident("_start"),
        ] ;
        "global directive"
    )]
    #[test_case(
        ".section .text",
        vec![
            token_dot(),
            token_ident("section"),
            token_dot(),
            token_ident("text"),
        ] ;
        "section directive"
    )]
    #[test_case(
        "_start:",
        vec![
            token_ident("_start"),
            token_colon(),
        ] ;
        "label definition"
    )]
    #[test_case(
        "li a0, 1",
        vec![
            token_ident("li"),
            token_ident("a0"),
            token_comma(),
            token_int("1"),
        ] ;
        "load immediate 1"
    )]
    #[test_case(
        "li a1, 32",
        vec![
            token_ident("li"),
            token_ident("a1"),
            token_comma(),
            token_int("32"),
        ] ;
        "load immediate 32"
    )]
    #[test_case(
        "li a2, 13",
        vec![
            token_ident("li"),
            token_ident("a2"),
            token_comma(),
            token_int("13"),
        ] ;
        "load immediate 13"
    )]
    #[test_case(
        "li a7, 64",
        vec![
            token_ident("li"),
            token_ident("a7"),
            token_comma(),
            token_int("64"),
        ] ;
        "load immediate 64"
    )]
    #[test_case(
        "la a1, helloworld",
        vec![
            token_ident("la"),
            token_ident("a1"),
            token_comma(),
            token_ident("helloworld"),
        ] ;
        "load address"
    )]
    #[test_case(
        "ecall",
        vec![token_ident("ecall")] ;
        "environment call"
    )]
    fn tokenize_returns_tokens(program: &str, expected: Vec<Token>) {
        let got = tokenize(program);
        assert_eq!(expected, got);
    }

    // Helper functions for creating tokens
    fn token_ident(lexeme: &str) -> Token {
        Token::new(TokenType::Identifier, lexeme)
    }

    fn token_int(lexeme: &str) -> Token {
        Token::new(TokenType::Integer, lexeme)
    }

    fn token_string(lexeme: &str) -> Token {
        Token::new(TokenType::String, lexeme)
    }

    fn token_comment(lexeme: &str) -> Token {
        Token::new(TokenType::Comment, lexeme)
    }

    fn token_comma() -> Token {
        Token::new(TokenType::Comma, ",")
    }

    fn token_colon() -> Token {
        Token::new(TokenType::Colon, ":")
    }

    fn token_dot() -> Token {
        Token::new(TokenType::Dot, ".")
    }

    fn token_minus() -> Token {
        Token::new(TokenType::Minus, "-")
    }

    fn token_lparen() -> Token {
        Token::new(TokenType::LParen, "(")
    }

    fn token_rparen() -> Token {
        Token::new(TokenType::RParen, ")")
    }
}
