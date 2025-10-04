#![allow(
    unused,
    clippy::cast_possible_truncation,
    clippy::needless_pass_by_value,
    clippy::unreadable_literal
)]
mod asm;
mod ast;
mod exe;
mod lexer;
mod machine;
mod parser;
pub mod prelude;

pub use exe::{build_exe, try_image_from_bytes};
