#![allow(clippy::cast_possible_truncation, clippy::unreadable_literal)]
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

pub type Address = u64;

#[derive(Debug, Default)]
pub struct State {
    /// Program counter
    pub pc: Address,

    /// Byte-addressable memory
    pub memory: Vec<u8>,

    /// General-purpose registers
    pub regs: Vec<u64>,

    /// Status flags (if architecture supports it)
    pub flags: Option<u64>,
}

impl State {
    /// Create a new state with the given number of registers and memory size.
    fn new(num_regs: usize, memory_size: usize) -> Self {
        Self {
            pc: 0,
            memory: vec![0u8; memory_size],
            regs: vec![0u64; num_regs],
            flags: None,
        }
    }

    #[must_use]
    /// Get the value of a register.
    pub fn get_reg(&self, index: usize) -> u64 {
        self.regs.get(index).copied().unwrap_or_default()
    }

    /// Set the value of a register.
    pub fn set_reg(&mut self, index: usize, value: u64) {
        if let Some(reg) = self.regs.get_mut(index) {
            *reg = value;
        }
    }

    /// Read a byte from memory.
    ///
    /// # Errors
    ///
    /// Returns an error if the address is out of bounds.
    pub fn read_u8(&self, addr: Address) -> Result<u8> {
        self.memory
            .get(addr as usize)
            .copied()
            .ok_or_else(|| anyhow!("memory out of bounds: {addr:#x}"))
    }

    /// Write a byte to memory.
    ///
    /// # Errors
    ///
    /// Returns an error if the address is out of bounds.
    pub fn write_u8(&mut self, addr: Address, value: u8) -> Result<()> {
        let byte = self
            .memory
            .get_mut(addr as usize)
            .ok_or_else(|| anyhow!("memory write out of bounds: {addr:#x}"))?;
        *byte = value;
        Ok(())
    }

    /// Read a 16-bit value from memory.
    ///
    /// # Errors
    ///
    /// Returns an error if the address is out of bounds.
    pub fn read_u16(&self, addr: Address) -> Result<u16> {
        let addr = addr as usize;
        let bytes: [u8; 2] = self
            .memory
            .get(addr..addr + 2)
            .ok_or_else(|| anyhow!("memory read out of bounds: {addr:#x}"))?
            .try_into()?;
        Ok(u16::from_le_bytes(bytes))
    }

    /// Write a 16-bit value to memory.
    ///
    /// # Errors
    ///
    /// Returns an error if the address is out of bounds.
    pub fn write_u16(&mut self, addr: Address, value: u16) -> Result<()> {
        let addr = addr as usize;
        let bytes = self
            .memory
            .get_mut(addr..addr + 2)
            .ok_or_else(|| anyhow!("memory write out of bounds: {addr:#x}"))?;
        bytes.copy_from_slice(&value.to_le_bytes());
        Ok(())
    }

    /// Read a 32-bit value from memory.
    ///
    /// # Errors
    ///
    /// Returns an error if the address is out of bounds.
    pub fn read_u32(&self, addr: Address) -> Result<u32> {
        let addr = addr as usize;
        let bytes: [u8; 4] = self
            .memory
            .get(addr..addr + 4)
            .ok_or_else(|| anyhow!("memory read out of bounds: {addr:#x}"))?
            .try_into()?;
        Ok(u32::from_le_bytes(bytes))
    }

    /// Write a 32-bit value to memory.
    ///
    /// # Errors
    ///
    /// Returns an error if the address is out of bounds.
    pub fn write_u32(&mut self, addr: Address, value: u32) -> Result<()> {
        let addr = addr as usize;
        let bytes = self
            .memory
            .get_mut(addr..addr + 4)
            .ok_or_else(|| anyhow!("memory write out of bounds: {addr:#x}"))?;
        bytes.copy_from_slice(&value.to_le_bytes());
        Ok(())
    }

    /// Read a 64-bit value from memory.
    ///
    /// # Errors
    ///
    /// Returns an error if the address is out of bounds.
    pub fn read_u64(&self, addr: Address) -> Result<u64> {
        let addr = addr as usize;
        let bytes: [u8; 8] = self
            .memory
            .get(addr..addr + 8)
            .ok_or_else(|| anyhow!("memory read out of bounds: {addr:#x}"))?
            .try_into()?;
        Ok(u64::from_le_bytes(bytes))
    }

    /// Write a 64-bit value to memory.
    ///
    /// # Errors
    ///
    /// Returns an error if the address is out of bounds.
    pub fn write_u64(&mut self, addr: Address, value: u64) -> Result<()> {
        let addr = addr as usize;
        let bytes = self
            .memory
            .get_mut(addr..addr + 8)
            .ok_or_else(|| anyhow!("memory write out of bounds: {addr:#x}"))?;
        bytes.copy_from_slice(&value.to_le_bytes());
        Ok(())
    }
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
    /// Create a new machine with the given memory size.
    pub fn new(memory_size: usize) -> Self {
        Self {
            state: State::new(I::NUM_REGISTERS, memory_size),
            _phantom: PhantomData,
        }
    }

    /// Step the machine by one instruction
    ///
    /// # Errors
    ///
    /// Returns an error if the instruction is illegal or if there is an error executing the instruction.
    pub fn step(&mut self) -> Result<()> {
        // Fetch
        let pc = self.state.pc as usize;
        let bytes = self
            .state
            .memory
            .get(pc..)
            .ok_or(anyhow!("memory read out of bounds: {pc:#x}"))?;

        // Decode
        let spec =
            I::lookup_spec(bytes).ok_or_else(|| anyhow!("illegal instruction at pc={pc:#x}"))?;

        // Execute
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
    fn machine_new_returns_initialized_machine() {
        let machine = Machine::<TestISA>::new(1024);

        let want = vec![0u64; 1];
        let got = machine.state.regs.clone();
        assert_eq!(want, got, "Registers should be initialized to zero");

        let want_pc = 0;
        let got_pc = machine.state.pc;
        assert_eq!(
            want_pc, got_pc,
            "Program counter should be initialized to zero"
        );

        let want_memory = vec![0u8; 1024];
        let got_memory = machine.state.memory.clone();
        assert_eq!(
            want_memory, got_memory,
            "Memory should be initialized to zero"
        );

        let want_flags = None;
        let got_flags = machine.state.flags;
        assert_eq!(want_flags, got_flags, "Flags should be initialized to None");
    }

    #[test]
    fn machine_step_executes_one_instruction() {
        let mut machine = Machine::<TestISA>::new(1024);
        machine.state.memory[0] = 0x00; // "halt" instruction

        machine.step().unwrap();

        let want_pc = 1;
        let got_pc = machine.state.pc;
        assert_eq!(
            want_pc, got_pc,
            "Program counter should be incremented after executing a HALT instruction"
        );
    }

    #[test]
    fn get_reg_fn_returns_register_value() {
        let mut state = State::new(1, 32);
        state.regs[0] = 42;

        let want = 42;
        let got = state.get_reg(0);
        assert_eq!(want, got, "Register value should be returned");
    }

    #[test]
    fn set_reg_fn_sets_register_value() {
        let mut state = State::new(1, 1);

        state.set_reg(0, 42);

        let want = 42;
        let got = state.regs[0];
        assert_eq!(want, got, "Register value should be set");
    }

    #[test]
    fn read_u8_fn_reads_byte_from_memory_at_given_address() {
        let mut state = State::new(1, 1);
        state.memory[0] = 42;

        let want = 42;
        let got = state.read_u8(0).unwrap();
        assert_eq!(want, got, "byte should be read from memory");
    }

    #[test]
    fn write_u8_fn_writes_byte_to_memory_at_given_address() {
        let mut state = State::new(1, 1);

        state
            .write_u8(0, 42)
            .expect("Failed to write byte to memory");

        let want = 42;
        let got = state.read_u8(0).unwrap();
        assert_eq!(
            want, got,
            "byte should be written to memory in little-endian"
        );
    }

    #[test]
    fn read_u16_fn_reads_u16_from_memory_at_given_address() {
        let mut state = State::new(1, 2);
        state.memory[0] = 0xcd;
        state.memory[1] = 0xab;

        let want = 0xabcd;
        let got = state.read_u16(0).unwrap();
        assert_eq!(want, got, "u16 should be read from memory");
    }

    #[test]
    fn write_u16_fn_writes_u16_to_memory_at_given_address() {
        let mut state = State::new(1, 2);

        state
            .write_u16(0, 0xabcd)
            .expect("Failed to write u16 to memory");

        let want = vec![0xcd, 0xab];
        let got = state.memory;
        assert_eq!(
            want, got,
            "u16 should be written to memory in little-endian"
        );
    }

    #[test]
    fn read_u32_fn_reads_u32_from_memory_at_given_address() {
        let mut state = State::new(1, 4);
        // Little-endian: 0xdeadbeef = [0xef, 0xbe, 0xad, 0xde]
        state.memory[0] = 0xef;
        state.memory[1] = 0xbe;
        state.memory[2] = 0xad;
        state.memory[3] = 0xde;

        let want = 0xdeadbeef;
        let got = state.read_u32(0).unwrap();
        assert_eq!(want, got, "u32 should be read from memory in little-endian");
    }

    #[test]
    fn write_u32_fn_writes_u32_to_memory_at_given_address() {
        let mut state = State::new(1, 4);

        state
            .write_u32(0, 0xdeadbeef)
            .expect("Failed to write u32 to memory");

        let want = vec![0xef, 0xbe, 0xad, 0xde];
        let got = state.memory;
        assert_eq!(
            want, got,
            "u32 should be written to memory in little-endian"
        );
    }

    #[test]
    fn read_u64_fn_reads_u64_from_memory_at_given_address() {
        let mut state = State::new(1, 8);
        // Little-endian: 0xdeadbeefcafebabe = [0xbe, 0xba, 0xfe, 0xca, 0xef, 0xbe, 0xad, 0xde]
        state.memory[0] = 0xbe;
        state.memory[1] = 0xba;
        state.memory[2] = 0xfe;
        state.memory[3] = 0xca;
        state.memory[4] = 0xef;
        state.memory[5] = 0xbe;
        state.memory[6] = 0xad;
        state.memory[7] = 0xde;

        let want = 0xdeadbeefcafebabe;
        let got = state.read_u64(0).unwrap();
        assert_eq!(want, got, "u64 should be read from memory in little-endian");
    }

    #[test]
    fn write_u64_fn_writes_u64_to_memory_at_given_address() {
        let mut state = State::new(1, 8);

        state
            .write_u64(0, 0xdeadbeefcafebabe)
            .expect("Failed to write u64 to memory");

        let want = vec![0xbe, 0xba, 0xfe, 0xca, 0xef, 0xbe, 0xad, 0xde];
        let got = state.memory;
        assert_eq!(
            want, got,
            "u64 should be written to memory in little-endian"
        );
    }
}
