use std::{
    iter::{Peekable, from_fn},
    str::Chars,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Illegal(char),
    Dot,
    Hash,
    Colon,
    Comma,
    LParen,
    RParen,
    DecLiteral(String),
    HexLiteral(String),
    Identifier(String),
}

pub struct Tokenizer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Tokenizer<'a> {
    #[must_use]
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
        }
    }
}

impl Tokenizer<'_> {
    fn skip_whitespace_and_comments(&mut self) {
        loop {
            // skip whitespace
            while self.chars.next_if(char::is_ascii_whitespace).is_some() {}

            // skip comments
            if self.chars.next_if_eq(&';').is_some() {
                while self.chars.next_if(|&c| c != '\n').is_some() {}
            } else {
                break;
            }
        }
    }

    fn read_identifier(&mut self) -> Token {
        let ident = from_fn(|| self.chars.next_if(char::is_ascii_alphanumeric)).collect();

        Token::Identifier(ident)
    }

    fn read_dec_literal(&mut self) -> Token {
        let ident: String = from_fn(|| self.chars.next_if(char::is_ascii_digit)).collect();

        Token::DecLiteral(ident)
    }

    fn read_hex_literal(&mut self) -> Token {
        self.chars.next(); // consume $
        let digits: String = from_fn(|| self.chars.next_if(char::is_ascii_hexdigit)).collect();
        if digits.is_empty() {
            Token::Illegal('$')
        } else {
            Token::HexLiteral(digits)
        }
    }
}

impl Iterator for Tokenizer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace_and_comments();
        let token = match self.chars.peek()? {
            ':' => {
                self.chars.next();
                Token::Colon
            }
            ',' => {
                self.chars.next();
                Token::Comma
            }
            '.' => {
                self.chars.next();
                Token::Dot
            }
            '#' => {
                self.chars.next();
                Token::Hash
            }
            '(' => {
                self.chars.next();
                Token::LParen
            }
            ')' => {
                self.chars.next();
                Token::RParen
            }
            '$' => self.read_hex_literal(),
            c if c.is_ascii_digit() => self.read_dec_literal(),
            c if c.is_ascii_alphabetic() => self.read_identifier(),
            c => {
                let c = *c;
                self.chars.next();
                Token::Illegal(c)
            }
        };
        Some(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("😀", &[illegal('😀')] ; "handles illegal tokens")]
    #[test_case("$", &[illegal('$')] ; "a bare $ is illegal")]
    #[test_case(",", &[comma()])]
    #[test_case("()", &[lparen(), rparen()])]
    #[test_case("; comments are skipped", &[])]
    #[test_case("foo", &[ident("foo")])]
    #[test_case("BAR", &[ident("BAR")])]
    #[test_case("$1234", &[hex("1234")] ; "can tokenize a hex literal")]
    #[test_case("1234", &[dec("1234")] ; "can tokenize a decimal literal")]
    #[test_case("#$10", &[hash(), hex("10")])]
    #[test_case(".word", &[dot(), ident("word")])]
    #[test_case("label:", &[ident("label"), colon()])]
    #[test_case(
        "LDA $10",
        &[ident("LDA"), hex("10")]
    )]
    #[test_case(
        "LDA #$10",
        &[ident("LDA"), hash(), hex("10")]
    )]
    #[test_case(
        "LDA #42",
        &[ident("LDA"), hash(), dec("42")]
    )]
    #[test_case(
        "LDA    $10",
        &[ident("LDA"), hex("10")]
        ; "skips extra whitespace"
    )]
    #[test_case(
        "LDA\n$10",
        &[ident("LDA"), hex("10")]
        ; "skips newlines"
    )]
    #[test_case(
        "LDA $10,X",
        &[
            ident("LDA"),
            hex("10"),
            comma(),
            ident("X"),
        ]
    )]
    #[test_case(
        "LDA ($10,X)",
        &[
            ident("LDA"),
            lparen(),
            hex("10"),
            comma(),
            ident("X"),
            rparen(),
        ]
    )]
    #[test_case(
        "LDA ($10),Y",
        &[
            ident("LDA"),
            lparen(),
            hex("10"),
            rparen(),
            comma(),
            ident("Y"),
        ]
    )]
    #[test_case(
        "LDA $10 ; a comment",
        &[
            ident("LDA"),
            hex("10"),
        ]
    )]
    #[test_case(
        "LDA ; comment\n $10 ; another comment",
        &[
            ident("LDA"),
            hex("10"),
        ]
    )]
    #[test_case(
        "LDA ; comment\n $10 ; another comment\nSTA $1000",
        &[
            ident("LDA"),
            hex("10"),
            ident("STA"),
            hex("1000"),
        ]
    )]
    fn tokenize_returns_tokens(program: &str, expected: &[Token]) {
        let tokenizer = Tokenizer::new(program);
        let got = tokenizer.collect::<Vec<_>>();
        assert_eq!(expected, got);
    }

    fn ident(s: &str) -> Token {
        Token::Identifier(s.into())
    }

    fn dec(s: &str) -> Token {
        Token::DecLiteral(s.into())
    }

    fn hex(s: &str) -> Token {
        Token::HexLiteral(s.into())
    }

    fn colon() -> Token {
        Token::Colon
    }

    fn comma() -> Token {
        Token::Comma
    }

    fn dot() -> Token {
        Token::Dot
    }

    fn hash() -> Token {
        Token::Hash
    }

    fn lparen() -> Token {
        Token::LParen
    }

    fn rparen() -> Token {
        Token::RParen
    }

    fn illegal(c: char) -> Token {
        Token::Illegal(c)
    }
}
