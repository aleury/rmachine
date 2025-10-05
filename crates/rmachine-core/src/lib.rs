use std::marker::PhantomData;

use anyhow::Result;
use anyhow::anyhow;

pub trait InstructionSpec {
    fn mnemonic(&self) -> &str;
}

pub trait InstructionSet: Sized {
    /// Number of general purpose registers in the instruction set.
    const NUM_REGISTERS: usize;

    /// Instruction specification.
    type Spec: InstructionSpec;

    type Context<'a>;

    /// Lookup the instruction specification for the given bytes.
    fn lookup_spec(bytes: &[u8]) -> Option<&'static Self::Spec>;

    /// Create a new context for the given instruction specification.
    fn create_context(state: &mut State) -> Self::Context<'_>;

    /// Execute the instruction with the given mnemonic.
    ///
    /// # Errors
    ///
    /// Returns an error if the instruction is not supported or if the mnemonic is invalid.
    fn execute(mnemonic: &str, ctx: &mut Self::Context<'_>) -> Result<()>;
}

#[derive(Debug, Default)]
pub struct State {
    /// Program counter
    pub pc: u64,

    /// Byte-addressable memory
    pub memory: Vec<u8>,

    /// General-purpose registers
    pub regs: Vec<u64>,

    /// Status flags (if architecture supports it)
    pub flags: Option<u64>,
}

#[derive(Debug, Default)]
pub struct Machine<I: InstructionSet> {
    pub state: State,
    _phantom: PhantomData<I>,
}

impl<I: InstructionSet> Machine<I>
where
    <I as InstructionSet>::Spec: 'static,
{
    #[must_use]
    pub fn new(memory_size: usize) -> Self {
        Self {
            state: State {
                pc: 0,
                memory: vec![0u8; memory_size],
                regs: vec![0u64; I::NUM_REGISTERS],
                flags: None,
            },
            _phantom: PhantomData,
        }
    }

    /// Step the machine by one instruction
    ///
    /// # Errors
    ///
    /// Returns an error if the instruction is illegal or if there is an error executing the instruction.
    pub fn step(&mut self) -> Result<()> {
        // 1. Fetch
        let pc = self.state.pc.try_into()?;
        let bytes = &self.state.memory[pc..];

        // 2. Decode
        let spec =
            I::lookup_spec(bytes).ok_or_else(|| anyhow!("illegal instruction at pc={pc:#x}"))?;

        // 3. Execute
        let mut ctx = I::create_context(&mut self.state);
        I::execute(spec.mnemonic(), &mut ctx)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::bail;

    use super::*;

    struct TestISA;

    struct TestSpec {
        mnemonic: &'static str,
        opcode: u8,
    }

    impl InstructionSpec for TestSpec {
        fn mnemonic(&self) -> &str {
            self.mnemonic
        }
    }

    const TEST_INSTRUCTIONS: &[TestSpec] = &[TestSpec {
        mnemonic: "halt",
        opcode: 0x00,
    }];

    struct TestContext<'a> {
        state: &'a mut State,
    }

    impl TestContext<'_> {
        fn advance_pc(&mut self) {
            self.state.pc += 1;
        }
    }

    impl InstructionSet for TestISA {
        const NUM_REGISTERS: usize = 1;

        type Spec = TestSpec;

        type Context<'a> = TestContext<'a>;

        fn lookup_spec(bytes: &[u8]) -> Option<&'static Self::Spec> {
            TEST_INSTRUCTIONS
                .iter()
                .find(|spec| spec.opcode == bytes[0])
        }

        fn create_context(state: &mut State) -> Self::Context<'_> {
            TestContext { state }
        }

        fn execute<'a>(mnemonic: &str, ctx: &mut Self::Context<'_>) -> Result<()> {
            match mnemonic {
                "halt" => {
                    ctx.advance_pc();
                    Ok(())
                }
                _ => bail!("unknown instruction {mnemonic:?}"),
            }
        }
    }

    #[test]
    fn new_returns_initialized_machine() {
        let machine = Machine::<TestISA>::new(1024);

        let want = vec![0u64; 1];
        let got = machine.state.regs.clone();
        assert_eq!(got, want, "Registers should be initialized to zero");

        let want_pc = 0;
        let got_pc = machine.state.pc;
        assert_eq!(
            got_pc, want_pc,
            "Program counter should be initialized to zero"
        );

        let want_memory = vec![0u8; 1024];
        let got_memory = machine.state.memory.clone();
        assert_eq!(
            got_memory, want_memory,
            "Memory should be initialized to zero"
        );

        let want_flags = None;
        let got_flags = machine.state.flags;
        assert_eq!(got_flags, want_flags, "Flags should be initialized to None");
    }

    #[test]
    fn step_executes_one_instruction() {
        let mut machine = Machine::<TestISA>::new(1024);
        machine.state.memory[0] = 0x00; // "halt" instruction

        machine.step().unwrap();

        let want_pc = 1;
        let got_pc = machine.state.pc;
        assert_eq!(
            got_pc, want_pc,
            "Program counter should be incremented after executing a HALT instruction"
        );
    }
}
