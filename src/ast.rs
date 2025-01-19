use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub struct Program {
    pub lines: Vec<Line>,
}

#[derive(Debug, PartialEq)]
pub enum Line {
    Label(String),
    Directive(Directive),
    Instruction(Instruction),
}

#[derive(Debug, PartialEq)]
pub struct Identifier(pub String);

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, PartialEq)]
pub enum Directive {
    Ascii(String),
    Global(String),
    Section(String),
}

#[derive(Debug, PartialEq)]
pub struct Instruction {
    pub name: String,
    pub operands: Vec<Operand>,
}

#[derive(Debug, PartialEq)]
pub enum Operand {
    Immediate(u32),
    Register(String),
    Symbol(String),
}
