#![allow(clippy::cast_possible_truncation, clippy::unreadable_literal)]
use std::marker::PhantomData;

use anyhow::Result;
use anyhow::anyhow;

pub trait Cpu: Default {
    type ISA: InstructionSet<Cpu = Self>;

    /// Program counter.
    fn pc(&self) -> usize;

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

pub type Address = usize;

/// Byte-addressable memory.
#[derive(Debug, Default)]
pub struct Memory(Vec<u8>);

impl Memory {
    #[must_use]
    pub fn new(memory_size: usize) -> Self {
        Self(vec![0u8; memory_size])
    }

    /// Read a byte from memory.
    ///
    /// # Errors
    ///
    /// Returns an error if the address is out of bounds.
    pub fn read_u8(&self, addr: Address) -> Result<u8> {
        self.0
            .get(addr)
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
            .0
            .get_mut(addr)
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
        let bytes: [u8; 2] = self
            .0
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
        let bytes = self
            .0
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
        let bytes: [u8; 4] = self
            .0
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
        let bytes = self
            .0
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
        let bytes: [u8; 8] = self
            .0
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
        let bytes = self
            .0
            .get_mut(addr..addr + 8)
            .ok_or_else(|| anyhow!("memory write out of bounds: {addr:#x}"))?;
        bytes.copy_from_slice(&value.to_le_bytes());
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct Machine<I: InstructionSet> {
    pub cpu: I::Cpu,
    pub memory: Memory,
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
            cpu: I::Cpu::default(),
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
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestISA;

    #[derive(Debug, Default)]
    struct TestCpu {
        reg: u8,
        pc: u16,
    }

    struct TestSpec {
        opcode: u8,
        execute_fn: fn(&mut TestContext) -> Result<()>,
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

        fn pc(&self) -> usize {
            self.pc as usize
        }

        fn step(&mut self, memory: &mut Memory) -> Result<()> {
            let pc = self.pc();
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
        pub fn halt(ctx: &mut TestContext) -> Result<()> {
            ctx.cpu.pc += 1;
            Ok(())
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

    #[test]
    fn read_u8_fn_reads_byte_from_memory_at_given_address() {
        let memory = Memory(vec![42]);

        let want = 42;
        let got = memory.read_u8(0).unwrap();
        assert_eq!(want, got, "byte should be read from memory");
    }

    #[test]
    fn write_u8_fn_writes_byte_to_memory_at_given_address() {
        let mut state = Memory::new(1);

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
        let memory = Memory(vec![0xcd, 0xab]);

        let want = 0xabcd;
        let got = memory.read_u16(0).unwrap();
        assert_eq!(want, got, "u16 should be read from memory");
    }

    #[test]
    fn write_u16_fn_writes_u16_to_memory_at_given_address() {
        let mut memory = Memory::new(2);

        memory
            .write_u16(0, 0xabcd)
            .expect("Failed to write u16 to memory");

        let want = vec![0xcd, 0xab];
        let got = memory.0;
        assert_eq!(
            want, got,
            "u16 should be written to memory in little-endian"
        );
    }

    #[test]
    fn read_u32_fn_reads_u32_from_memory_at_given_address() {
        let memory = Memory(vec![0xef, 0xbe, 0xad, 0xde]);

        let want = 0xdeadbeef;
        let got = memory.read_u32(0).unwrap();
        assert_eq!(want, got, "u32 should be read from memory in little-endian");
    }

    #[test]
    fn write_u32_fn_writes_u32_to_memory_at_given_address() {
        let mut memory = Memory::new(4);

        memory
            .write_u32(0, 0xdeadbeef)
            .expect("Failed to write u32 to memory");

        let want = vec![0xef, 0xbe, 0xad, 0xde];
        let got = memory.0;
        assert_eq!(
            want, got,
            "u32 should be written to memory in little-endian"
        );
    }

    #[test]
    fn read_u64_fn_reads_u64_from_memory_at_given_address() {
        let memory = Memory(vec![0xbe, 0xba, 0xfe, 0xca, 0xef, 0xbe, 0xad, 0xde]);

        let want = 0xdeadbeefcafebabe;
        let got = memory.read_u64(0).unwrap();
        assert_eq!(want, got, "u64 should be read from memory in little-endian");
    }

    #[test]
    fn write_u64_fn_writes_u64_to_memory_at_given_address() {
        let mut memory = Memory::new(8);

        memory
            .write_u64(0, 0xdeadbeefcafebabe)
            .expect("Failed to write u64 to memory");

        let want = vec![0xbe, 0xba, 0xfe, 0xca, 0xef, 0xbe, 0xad, 0xde];
        let got = memory.0;
        assert_eq!(
            want, got,
            "u64 should be written to memory in little-endian"
        );
    }
}
