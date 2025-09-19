//! The rmachine prelude - commonly used types and traits.
//!
//! This module provides a convenient way to import the most commonly used types
//! from the rmachine crate. Rather than importing each type individually, you can
//! use a glob import of the prelude:
//!
//! ```
//! use rmachine::prelude::*;
//! ```
//!
//! # Contents
//!
//! The prelude includes:
//! - Core CPU types: [`Instruction`], [`Opcode`], [`Reg`], [`Word`]
//! - Machine emulation: [`Machine`], [`TermSys`]
//!
//! These types provide everything needed to work with R-Machine assembly code,
//! emulate CPU execution, and interact with the system.

pub use crate::asm::{Instruction, Opcode, Reg, Word};
pub use crate::machine::{Machine, TermSys};
