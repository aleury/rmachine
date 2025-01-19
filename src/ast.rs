#[derive(Debug, PartialEq)]
pub struct Program {
    pub lines: Vec<Line>,
}

#[derive(Debug, PartialEq)]
pub enum Line {
    Label(Identifier),
    Instruction {
        name: Identifier,
        operands: Vec<Operand>,
    },
}

#[derive(Debug, PartialEq)]
pub struct Identifier(pub String);

#[derive(Debug, PartialEq)]
pub enum Operand {
    Register(String),
    Immediate(u32),
}
