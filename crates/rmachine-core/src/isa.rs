use anyhow::Result;

use crate::memory::Memory;

pub trait Cpu: Default {
    /// Instruction set for the CPU.
    type ISA: InstructionSet<Cpu = Self>;

    /// Step the CPU by one instruction.
    ///
    /// This function fetches the next instruction from memory, decodes it, and executes it.
    ///
    /// # Errors
    ///
    /// Returns an error if the instruction is not supported or if the mnemonic is invalid.
    fn step(&mut self, memory: &mut Memory) -> Result<()>;
}

pub trait InstructionSet: Sized {
    /// CPU type for the instruction set.
    type Cpu: Cpu<ISA = Self>;

    /// Instruction specification.
    type Spec;

    /// Execution context.
    type Context<'a>;
}
