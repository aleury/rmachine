#![allow(clippy::cast_possible_truncation)]
use std::marker::PhantomData;

use anyhow::Result;

use crate::{
    isa::{Cpu, InstructionSet, StepResult},
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
    pub fn step(&mut self) -> Result<StepResult> {
        self.cpu.step(&mut self.memory)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;

    struct TestISA;

    #[derive(Debug, Default)]
    struct TestCpu {
        reg: u8,
        pc: u16,
    }

    struct TestSpec {
        opcode: u8,
        execute_fn: fn(&mut TestContext) -> Result<StepResult>,
    }

    struct TestContext<'a> {
        cpu: &'a mut TestCpu,
    }

    impl InstructionSet for TestISA {
        type Cpu = TestCpu;

        type Spec = TestSpec;

        type Context<'a> = TestContext<'a>;
    }

    impl Cpu for TestCpu {
        type ISA = TestISA;

        fn step(&mut self, memory: &mut Memory) -> Result<StepResult> {
            let pc = self.pc as usize;
            let opcode = memory.read_u8(pc)?;

            let spec = TEST_INSTRUCTIONS
                .iter()
                .find(|s| s.opcode == opcode)
                .ok_or_else(|| anyhow!("illegal instruction at pc={pc:#x}"))?;

            let mut ctx = TestContext { cpu: self };

            (spec.execute_fn)(&mut ctx)
        }
    }

    const TEST_INSTRUCTIONS: &[TestSpec] = &[TestSpec {
        opcode: 0x00,
        execute_fn: TestISA::halt,
    }];

    impl TestISA {
        #[allow(clippy::unnecessary_wraps)]
        pub fn halt(ctx: &mut TestContext) -> Result<StepResult> {
            ctx.cpu.pc += 1;
            Ok(StepResult::Halt)
        }
    }

    #[test]
    fn machine_new_returns_initialized_machine() {
        let machine = Machine::<TestISA>::new(1024);

        let want_reg = 0u8;
        let got_reg = machine.cpu.reg;
        assert_eq!(want_reg, got_reg, "Register should be initialized to zero");

        let want_pc = 0;
        let got_pc = machine.cpu.pc;
        assert_eq!(
            want_pc, got_pc,
            "Program counter should be initialized to zero"
        );

        let want_memory = vec![0; 1024];
        let got_memory = machine.memory.0.clone();
        assert_eq!(
            want_memory, got_memory,
            "Memory should be initialized to zero"
        );
    }

    #[test]
    fn machine_step_executes_one_instruction() {
        let mut machine = Machine::<TestISA>::new(1024);
        machine.memory.0[0] = 0x00; // "halt" instruction

        machine.step().unwrap();

        let want_pc = 1;
        let got_pc = machine.cpu.pc;
        assert_eq!(
            want_pc, got_pc,
            "Program counter should be incremented after executing a `halt` instruction"
        );
    }
}
