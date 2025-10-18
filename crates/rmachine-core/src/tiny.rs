use crate::{
    isa::{Cpu, InstructionSet},
    memory::{Address, Memory},
};
use anyhow::{Result, anyhow};

/// Represents the Tiny ISA.
pub struct TinyISA;

/// Represents the context for executing Tiny instructions.
pub struct TinyContext<'a> {
    cpu: &'a mut TinyCpu,
    memory: &'a mut Memory,
}

impl TinyContext<'_> {
    /// Fetches a byte from memory and advances the program counter.
    ///
    /// # Errors
    ///
    /// May return an error if the memory read operation fails.
    pub fn fetch_byte(&mut self) -> Result<u8> {
        self.cpu.next(self.memory)
    }
}

#[derive(Debug, Default)]
pub struct TinyCpu {
    pub(crate) a: u8,
    pub(crate) x: u8,
    pub(crate) pc: u16,
}

impl TinyCpu {
    fn next(&mut self, memory: &mut Memory) -> Result<u8> {
        let byte = memory.read_u8(self.pc as Address)?;
        self.pc += 1;
        Ok(byte)
    }
}

pub struct TinySpec {
    opcode: u8,
    execute: fn(&mut TinyContext) -> Result<()>,
}

impl InstructionSet for TinyISA {
    type Cpu = TinyCpu;

    type Spec = TinySpec;

    type Context<'a> = TinyContext<'a>;
}

impl Cpu for TinyCpu {
    type ISA = TinyISA;

    fn step(&mut self, memory: &mut Memory) -> Result<()> {
        let pc = self.pc as usize;
        let opcode = self.next(memory)?;

        let spec = INSTRUCTIONS
            .iter()
            .find(|s| s.opcode == opcode)
            .ok_or_else(|| anyhow!("illegal instruction {opcode:#x} at pc={pc:#x}"))?;

        let mut ctx = TinyContext { cpu: self, memory };

        (spec.execute)(&mut ctx)
    }

    fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.pc = 0;
    }
}

const INSTRUCTIONS: &[TinySpec] = &[
    TinySpec {
        opcode: 0x00,
        execute: TinyISA::nop,
    },
    TinySpec {
        opcode: 0x01,
        execute: TinyISA::lda_imm,
    },
    TinySpec {
        opcode: 0x02,
        execute: TinyISA::lda_zp,
    },
    TinySpec {
        opcode: 0x03,
        execute: TinyISA::sta_zp,
    },
    TinySpec {
        opcode: 0x04,
        execute: TinyISA::ldx_imm,
    },
    TinySpec {
        opcode: 0x05,
        execute: TinyISA::ldx_zp,
    },
    TinySpec {
        opcode: 0x06,
        execute: TinyISA::inx,
    },
    TinySpec {
        opcode: 0x07,
        execute: TinyISA::stx_zp,
    },
];

impl TinyISA {
    #[allow(clippy::unnecessary_wraps)]
    /// No operation.
    ///
    /// # Errors
    ///
    /// This function never returns an error.
    pub fn nop(_ctx: &mut TinyContext) -> Result<()> {
        Ok(())
    }

    /// Load immediate value into register.
    ///
    /// # Errors
    ///
    /// May return an error if the memory read operation fails.
    pub fn lda_imm(ctx: &mut TinyContext) -> Result<()> {
        ctx.cpu.a = ctx.fetch_byte()?;
        Ok(())
    }

    /// Load value from zero page address into register A.
    ///
    /// # Errors
    ///
    /// May return an error if the memory read operation fails.
    pub fn lda_zp(ctx: &mut TinyContext) -> Result<()> {
        let addr = ctx.fetch_byte()? as Address;
        ctx.cpu.a = ctx.memory.read_u8(addr)?;
        Ok(())
    }

    /// Store value of register A into memory at the given zero page address.
    ///
    /// # Errors
    ///
    /// May return an error if the memory write operation fails.
    pub fn sta_zp(ctx: &mut TinyContext) -> Result<()> {
        let addr = ctx.fetch_byte()? as Address;
        ctx.memory.write_u8(addr, ctx.cpu.a)
    }

    /// Load immediate value into register X.
    ///
    /// # Errors
    ///
    /// May return an error if the memory read operation fails.
    pub fn ldx_imm(ctx: &mut TinyContext) -> Result<()> {
        ctx.cpu.x = ctx.fetch_byte()?;
        Ok(())
    }

    /// Load value from zero page address into register X.
    ///
    /// # Errors
    ///
    /// May return an error if the memory write operation fails.
    pub fn ldx_zp(ctx: &mut TinyContext) -> Result<()> {
        let addr = ctx.fetch_byte()? as Address;
        ctx.cpu.x = ctx.memory.read_u8(addr)?;
        Ok(())
    }

    /// Increment value of register X by one.
    ///
    /// # Errors
    ///
    /// Will not return an error.
    pub fn inx(ctx: &mut TinyContext) -> Result<()> {
        ctx.cpu.x = ctx.cpu.x.wrapping_add(1);
        Ok(())
    }

    /// Store value of register X at the given zero page address.
    ///
    /// # Errors
    ///
    /// May return an error if the memory write operation fails.
    pub fn stx_zp(ctx: &mut TinyContext) -> Result<()> {
        let addr = ctx.fetch_byte()? as Address;
        ctx.memory.write_u8(addr, ctx.cpu.x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset_clears_the_registers_on_the_cpu() {
        let mut cpu = TinyCpu {
            a: 0x42,
            x: 0x20,
            pc: 12,
        };

        cpu.reset();

        assert_eq!(cpu.a, 0);
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.pc, 0);
    }

    #[test]
    fn nop_does_nothing() {
        let mut cpu = TinyCpu {
            a: 0x42,
            x: 0,
            pc: 0,
        };
        let mut memory = Memory::new(256);
        memory.load(0, &[0x00]).unwrap();

        cpu.step(&mut memory).unwrap();

        assert_eq!(cpu.a, 0x42);
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.pc, 1);
    }

    #[test]
    fn lda_imm_fn_loads_immediate_value_into_register() {
        let mut cpu = TinyCpu { a: 0, x: 0, pc: 0 };
        let mut memory = Memory::new(256);
        memory.load(0, &[0x01, 0x42]).unwrap();

        cpu.step(&mut memory).unwrap();

        assert_eq!(cpu.a, 0x42);
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.pc, 2);
    }

    #[test]
    fn lda_zp_fn_loads_byte_from_zero_page_address() {
        let mut cpu = TinyCpu { a: 0, x: 0, pc: 0 };
        let mut memory = Memory::new(256);
        memory.write_u8(0x20, 0x42).unwrap(); // store value at zero page address
        memory.load(0, &[0x02, 0x20]).unwrap(); // load value from memory into register

        cpu.step(&mut memory).unwrap();

        assert_eq!(cpu.a, 0x42);
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.pc, 2);
    }

    #[test]
    fn sta_zp_fn_stores_register_value_into_memory_at_zero_page_address() {
        let mut cpu = TinyCpu {
            a: 0x42,
            x: 0,
            pc: 0,
        };
        let mut memory = Memory::new(256);
        memory.load(0, &[0x03, 0x20]).unwrap();

        cpu.step(&mut memory).unwrap();

        assert_eq!(cpu.a, 0x42);
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.pc, 2);
        assert_eq!(memory.read_u8(0x20).unwrap(), 0x42);
    }

    #[test]
    fn ldx_imm_fn_loads_immediate_value_into_register() {
        let mut cpu = TinyCpu { a: 0, x: 0, pc: 0 };
        let mut memory = Memory::new(256);
        memory.load(0, &[0x04, 0x42]).unwrap();

        cpu.step(&mut memory).unwrap();

        assert_eq!(cpu.a, 0);
        assert_eq!(cpu.x, 0x42);
        assert_eq!(cpu.pc, 2);
    }

    #[test]
    fn ldx_zp_fn_loads_byte_from_zero_page_address() {
        let mut cpu = TinyCpu { a: 0, x: 0, pc: 0 };
        let mut memory = Memory::new(256);
        memory.write_u8(0x20, 0x42).unwrap(); // store value at zero page address
        memory.load(0, &[0x05, 0x20]).unwrap(); // load value from memory into register

        cpu.step(&mut memory).unwrap();

        assert_eq!(cpu.a, 0);
        assert_eq!(cpu.x, 0x42);
        assert_eq!(cpu.pc, 2);
    }

    #[test]
    fn inx_increments_register_x_by_one() {
        let mut cpu = TinyCpu { a: 0, x: 0, pc: 0 };
        let mut memory = Memory::new(256);
        memory.load(0, &[0x06]).unwrap();

        cpu.step(&mut memory).unwrap();

        assert_eq!(cpu.a, 0);
        assert_eq!(cpu.x, 1);
        assert_eq!(cpu.pc, 1);
    }

    #[test]
    fn stx_zp_fn_stores_register_value_into_memory_at_zero_page_address() {
        let mut cpu = TinyCpu {
            a: 0,
            x: 0x42,
            pc: 0,
        };
        let mut memory = Memory::new(256);
        memory.load(0, &[0x07, 0x20]).unwrap();

        cpu.step(&mut memory).unwrap();

        assert_eq!(cpu.a, 0);
        assert_eq!(cpu.x, 0x42);
        assert_eq!(cpu.pc, 2);
        assert_eq!(memory.read_u8(0x20).unwrap(), 0x42);
    }
}
