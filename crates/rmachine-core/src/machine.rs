#![allow(clippy::cast_possible_truncation)]
use std::marker::PhantomData;

use anyhow::Result;

use crate::{
    isa::{Cpu, InstructionSet},
    memory::Memory,
};

#[derive(Debug, Default)]
pub struct Machine<ISA: InstructionSet> {
    cpu: ISA::Cpu,
    memory: Memory,
    _phantom: PhantomData<ISA>,
}

impl<ISA: InstructionSet> Machine<ISA> {
    #[must_use]
    /// Create a new machine with the given memory size.
    pub fn new(memory_size: usize) -> Self {
        Self {
            cpu: ISA::Cpu::default(),
            memory: Memory::new(memory_size),
            _phantom: PhantomData,
        }
    }

    /// Step the machine by one instruction
    ///
    /// # Errors
    ///
    /// Returns an error if the instruction is illegal or if there is an error executing the instruction.
    pub fn step(&mut self) -> Result<()> {
        self.cpu.step(&mut self.memory)
    }

    /// Run the machine until an error occurs.
    ///
    /// # Errors
    ///
    /// Returns an error when an instruction fails (illegal opcode, memory fault, etc.)
    pub fn run(&mut self) -> Result<()> {
        loop {
            self.step()?;
        }
    }

    /// Load a program into memory at the given address.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an error writing to memory.
    pub fn load(&mut self, addr: usize, program: &[u8]) -> Result<()> {
        self.memory.load(addr, program)
    }
}

#[cfg(test)]
mod tests {
    use crate::tiny::TinyISA;

    use super::*;

    #[test]
    fn machine_new_returns_initialized_machine() {
        let machine = Machine::<TinyISA>::new(1024);

        assert_eq!(machine.cpu.a, 0);
        assert_eq!(machine.cpu.pc, 0);
        assert_eq!(machine.memory, Memory(vec![0; 1024]));
    }

    #[test]
    fn load_loads_bytes_into_memory_at_the_given_address() {
        let mut machine = Machine::<TinyISA>::new(4);
        machine.load(0, &[0x01, 42]).unwrap();

        assert_eq!(machine.memory, Memory(vec![0x01, 42, 0, 0]));
    }

    #[test]
    fn step_increments_pc_register_after_executing_nop_instruction() {
        let mut machine = Machine::<TinyISA>::new(2);
        machine.load(0, &[0x00]).unwrap();

        machine.step().unwrap();

        assert_eq!(machine.cpu.pc, 1);
    }

    #[test]
    fn run_executes_instructions_until_an_error_occurs() {
        let mut machine = Machine::<TinyISA>::new(256);
        machine.load(0, &[0x01, 42, 0xFF]).unwrap(); // lda immediate

        let result = machine.run();

        assert!(result.is_err());
        assert_eq!(machine.cpu.a, 42);
        assert_eq!(machine.cpu.pc, 3);
    }
}
