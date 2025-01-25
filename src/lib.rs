#![allow(
    unused,
    clippy::cast_possible_truncation,
    clippy::needless_pass_by_value,
    clippy::unreadable_literal
)]
mod asm;
mod ast;
mod lexer;
mod machine;
mod parser;
pub mod prelude;

pub use asm::build_exe;
