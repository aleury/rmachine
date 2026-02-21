#![allow(clippy::cast_possible_truncation)]
use std::collections::HashMap;
use std::fmt::{Display, Write};
use std::fs;
use std::ops::Div;
use std::panic;
use std::panic::AssertUnwindSafe;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use anyhow::Result;
use num_traits::ToPrimitive;

use crate::Memory;
use crate::exception::Exception;

#[derive(Debug, Clone)]
pub enum Mode {
    Implied,
    Immediate,
    ZeroPage,
    ZeroPageX,
    Absolute,
    AbsoluteX,
    Relative,
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub mnemonic: &'static str,
    pub mode: Mode,
    pub opcode: u8,
    pub bytes: u8,
    pub cycles: u8,
    pub execute: fn(&mut Machine, &mut Memory),
    pub test: fn(&mut Machine, &mut Memory),
}

pub type HandlerMap = HashMap<&'static str, fn(&mut Machine, &mut Memory)>;

#[derive(Debug, Default)]
pub struct Machine {
    pub pc: u16,
    pub registers: HashMap<&'static str, u8>,
    register_list: &'static [&'static str],
    instructions: HashMap<u8, &'static Instruction>,
    handlers: HandlerMap,
    cycles: u64,
    cycle_time_ns: u64,
    timer_ns: u64,
    pub exception: Option<Exception>,
}

#[derive(Debug, Default)]
pub struct MachineBuilder {
    pub registers: &'static [&'static str],
    pub instructions: &'static [Instruction],
    pub handlers: HandlerMap,
    pub frequency_hz: u64,
}

impl MachineBuilder {
    /// Returns a `Machine` configured according to the builder specification.
    ///
    /// # Panics
    ///
    /// Will panic if an instruction with the same opcode already exists.
    #[must_use]
    pub fn build(self) -> Machine {
        Machine {
            registers: {
                let mut registers: HashMap<&'static str, u8> = HashMap::new();
                for register in self.registers {
                    registers.insert(register, 0);
                }
                registers
            },
            register_list: self.registers,
            instructions: {
                let mut instructions = HashMap::new();
                for instruction in self.instructions {
                    let present = instructions.insert(instruction.opcode, instruction);
                    assert!(
                        present.is_none(),
                        "duplicate opcode: {:#0X}",
                        instruction.opcode
                    );
                }
                instructions
            },
            handlers: self.handlers,
            cycle_time_ns: 1_000_000_000_u64
                .checked_div(self.frequency_hz)
                .expect("frequency must be non-zero"),
            ..Default::default()
        }
    }
}

impl Machine {
    /// Clears the machine registers and sets PC = 0.
    ///
    /// The contents of memory are not affected.
    pub fn reset(&mut self) {
        self.pc = 0;
        for register in self.registers.values_mut() {
            *register = 0;
        }
    }

    /// Runs the machine continuously until an exception occurs.
    pub fn run(&mut self, memory: &mut Memory) {
        self.exception = None;
        while self.exception.is_none() {
            self.step(memory);
        }
    }

    /// Runs a single instruction on the machine.
    ///
    /// Unknown opcodes are ignored.
    pub fn step(&mut self, memory: &mut Memory) {
        self.exception = None;
        let pc_start = self.pc;
        let opcode = self.fetch8(memory);
        if let Some(instruction) = self.instructions.get(&opcode).copied() {
            (instruction.execute)(self, memory);
            self.wait_cycles(instruction.cycles);
        }
        if self.pc == pc_start {
            self.trap(Exception::HaltAndCatchFire);
        }
    }

    /// Loads and runs the given program to completion.
    ///
    /// # Panics
    ///
    /// If the program will not fit in the machine's memory.
    pub fn run_program(&mut self, memory: &mut Memory, program: &[u8]) {
        memory.load(0, program).expect("program too big");
        self.pc = 0;
        self.run(memory);
    }

    /// Returns the program counter.
    #[must_use]
    pub fn pc(&self) -> u16 {
        self.pc
    }

    /// Advances the program counter by the given amount.
    pub fn advance(&mut self, amount: u16) {
        self.pc = self.pc.wrapping_add(amount);
    }

    /// Returns a copy of the named register contents.
    ///
    /// # Panics
    ///
    /// Panics if the named register does not exist.
    #[must_use]
    pub fn reg(&self, reg_name: &str) -> u8 {
        self.registers
            .get(reg_name)
            .copied()
            .ok_or_else(|| format!("undefined register '{reg_name}'"))
            .unwrap()
    }

    /// Returns a mutable reference to the named register contents.
    ///
    /// # Panics
    ///
    /// Panics if the named register does not exist.
    pub fn reg_mut(&mut self, reg_name: &str) -> &mut u8 {
        self.registers
            .get_mut(reg_name)
            .ok_or_else(|| format!("undefined register '{reg_name}'"))
            .unwrap()
    }

    /// Sets the named register contents.
    ///
    /// # Panics
    ///
    /// Panics if the named register does not exist.
    pub fn set_reg(&mut self, reg_name: &'static str, value: u8) {
        self.registers
            .insert(reg_name, value)
            .ok_or_else(|| format!("undefined register '{reg_name}'"))
            .unwrap();
    }

    /// Clears the specified bit of register `reg_name`.
    pub fn clear_bit(&mut self, reg_name: &'static str, bit: u8) {
        let reg = self.reg_mut(reg_name);
        *reg &= !bit;
    }

    /// Sets the specified bit of register `reg_name`.
    pub fn set_bit(&mut self, reg_name: &'static str, bit: u8) {
        let reg = self.reg_mut(reg_name);
        *reg |= bit;
    }

    /// Returns true if the specified bit of `reg_name` is set.
    #[must_use]
    pub fn test_bit(&self, reg_name: &'static str, bit: u8) -> bool {
        self.reg(reg_name) & bit != 0
    }

    /// Returns the next byte from memory, advancing PC.
    pub fn fetch8(&mut self, memory: &mut Memory) -> u8 {
        let value = memory.get8(self.pc);
        self.advance(1);
        value
    }

    /// Returns the next (little-endian) word from memory, advancing PC.
    ///
    /// # Errors
    ///
    /// Returns an error if the address is out of bounds.
    pub fn fetch16(&mut self, memory: &mut Memory) -> u16 {
        let value = memory.get16(self.pc);
        self.advance(2);
        value
    }

    /// Loads binary file `path` at address `addr` and sets PC to `addr`.
    ///
    /// # Errors
    ///
    /// If reading the file fails.
    pub fn load_bin(
        &mut self,
        memory: &mut Memory,
        addr: u16,
        path: impl AsRef<Path>,
    ) -> Result<()> {
        let data = fs::read(path)?;
        memory.load(addr, &data)?;
        self.pc = addr;
        Ok(())
    }

    /// Sleeps long enough to slow the emulator down to roughly the machine's
    /// rated clock frequency.
    ///
    /// Also updates the cycle counter and cycle timer, used to report the
    /// actual speed achieved (by [`Self::speed_mhz`]).
    ///
    /// # Panics
    ///
    /// If the calculated sleep would be more than [`u64::MAX`] nanoseconds
    /// (around 600 years).
    pub fn wait_cycles(&mut self, cycles: u8) {
        let cycles = u64::from(cycles);
        let delay = cycles
            .checked_mul(self.cycle_time_ns)
            .expect("unreasonably long delay");
        sleep(Duration::from_nanos(delay));
        let (new_cycles, overflow) = self.cycles.overflowing_add(cycles);
        if overflow {
            self.cycles = 0;
            self.timer_ns = 0;
        } else {
            self.cycles = new_cycles;
            self.timer_ns = self.timer_ns.saturating_add(delay);
        }
    }

    /// Runs all instruction self-tests.
    ///
    /// The machine is reset before each test (clearing the registers, but not
    /// the memory).
    ///
    /// # Panics
    ///
    /// If a test fails.
    pub fn self_test(&mut self) {
        let opcodes: Vec<_> = self.instructions.keys().copied().collect();
        let mut memory = Memory::new(0x10_000);
        for opcode in opcodes {
            self.reset();
            let instr = &self.instructions[&opcode].clone();
            if panic::catch_unwind(AssertUnwindSafe(|| (instr.test)(self, &mut memory))).is_err() {
                eprintln!("{self}");
                panic!("opcode {:#04X} failed self-test", instr.opcode);
            }
        }
    }

    /// Reports the approximate speed achieved by the machine in MHz.
    ///
    /// This is based on the number of cycles executed since the last cycle
    /// counter reset, divided by the internal timer value.
    ///
    /// Emulator overhead is not accounted for, and is assumed to be negligible
    /// relative to the rated clock frequency.
    ///
    /// # Panics
    ///
    /// If either `self.cycle_time_ns` or `self.timer_ns` are unrepresentable as
    /// `f64`.
    #[must_use]
    pub fn speed_mhz(&self) -> f64 {
        fn safe_f64(x: u64) -> f64 {
            x.to_f64().expect(
                "u64 values are always representable as f64; loss of precision is okay here",
            )
        }
        let cycle_time = safe_f64(self.cycle_time_ns);
        if self.cycles == 0 {
            1_000_f64.div(cycle_time)
        } else {
            let cycles = safe_f64(self.cycles);
            let elapsed_sec = safe_f64(self.timer_ns).div(1000.0);
            cycles.div(elapsed_sec)
        }
    }

    /// Disassemble the next instruction.
    ///
    /// # Errors
    ///
    /// May return an error if unable to disassemble next instruction.
    #[must_use]
    pub fn disassemble_next(&self, memory: &mut Memory) -> Option<String> {
        let opcode = memory.get8(self.pc);
        let instruction = self.instructions.get(&opcode)?;
        let mut disassembly = String::from(instruction.mnemonic);
        match instruction.bytes {
            1 => {}
            2 => {
                let value = memory.get8(self.pc.wrapping_add(1));
                write!(disassembly, " {value:#04x}").ok()?;
            }
            3 => {
                let value = memory.get16(self.pc.wrapping_add(1));
                write!(disassembly, " {value:#06x}").ok()?;
            }
            x => unreachable!("invalid number of bytes: {x}"),
        }
        Some(disassembly)
    }

    pub fn signal(&mut self, memory: &mut Memory, signal: &'static str) {
        if let Some(handler) = self.handlers.get(signal) {
            (handler)(self, memory);
        }
    }

    pub fn trap(&mut self, x: Exception) {
        self.exception = Some(x);
    }
}

impl Display for Machine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\nPC   ")?;
        for reg in self.register_list {
            write!(f, "{reg:} ")?;
        }
        writeln!(f, "INST       SPD")?;
        write!(f, "{:04X} ", self.pc)?;
        for reg in self.register_list {
            write!(
                f,
                "{:02X} ",
                self.registers
                    .get(reg)
                    .expect("reg should have a hashmap entry")
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub const INSTRUCTIONS: &[Instruction] = &[
        Instruction {
            mnemonic: "BRK",
            mode: Mode::Implied,
            opcode: 0x00,
            bytes: 1,
            cycles: 2,
            execute: |m, _| m.trap(Exception::Break),
            test: |m, mem| {
                m.run_program(
                    mem,
                    &[
                        0x00, // 0x0000 BRK
                    ],
                );
                assert_eq!(m.pc, 0x0001, "wrong PC");
            },
        },
        Instruction {
            mnemonic: "JMP",
            mode: Mode::Absolute,
            opcode: 0x04,
            bytes: 3,
            cycles: 2,
            execute: |m, mem| m.pc = m.fetch16(mem),
            test: |m, mem| {
                m.run_program(
                    mem,
                    &[
                        0x04, 0x04, 0x00, // 0x0000 JMP $0004
                        0x00, //             0x0003 BRK
                        0x00, //             0x0004 BRK
                    ],
                );
                assert_eq!(m.pc, 0x0005, "wrong PC");
            },
        },
        Instruction {
            mnemonic: "LDA",
            mode: Mode::Immediate,
            opcode: 0x02,
            bytes: 2,
            cycles: 2,
            execute: |m, mem| {
                let op = m.fetch8(mem);
                m.set_reg("AC", op);
            },
            test: |m, mem| {
                m.run_program(
                    mem,
                    &[
                        0x02, 0xFF, // 0x0000 LDA #FF
                        0x00, //       0x0002 BRK
                    ],
                );
                assert_eq!(m.reg("AC"), 0xFF, "wrong AC");
            },
        },
        Instruction {
            mnemonic: "NOP",
            mode: Mode::Implied,
            opcode: 0x01,
            bytes: 1,
            cycles: 2,
            execute: |_, _| (),
            test: |m, mem| {
                m.run_program(
                    mem,
                    &[
                        0x01, // 0x0000 NOP
                        0x00, // 0x0001 BRK
                    ],
                );
                assert_eq!(m.pc, 0x0002, "wrong PC");
            },
        },
    ];

    fn new_tiny_machine() -> Machine {
        MachineBuilder {
            registers: &["AC", "XR", "YR"],
            instructions: INSTRUCTIONS,
            frequency_hz: 1_000_000,
            ..Default::default()
        }
        .build()
    }

    #[test]
    fn machine_self_tests_pass() {
        new_tiny_machine().self_test();
    }

    #[test]
    fn cycles_are_counted() {
        let mut m = new_tiny_machine();
        let mut mem = Memory::new(1024);
        m.run_program(
            &mut mem,
            &[
                0x01, // 0x0000 NOP (2 cycles)
                0x01, // 0x0001 NOP (2 cycles)
                0x00, // 0x0002 BRK (2 cycles)
            ],
        );
        assert_eq!(m.cycles, 6);
    }

    #[test]
    fn bit_methods_work_correctly() {
        const BIT_0: u8 = 0b0000_0001;
        let mut m = new_tiny_machine();
        assert!(!m.test_bit("AC", BIT_0), "0 bit misreported as 1");
        m.set_bit("AC", BIT_0);
        assert!(m.test_bit("AC", BIT_0), "bit not set");
        m.clear_bit("AC", BIT_0);
        assert!(!m.test_bit("AC", BIT_0), "bit not cleared");
    }

    #[test]
    fn speed_mhz_fn_calculates_speed_correctly() {
        fn close_enough(x: f64, y: f64) {
            assert!((x - y).abs() < 0.1, "want {y:.2}, got {x:.2}");
        }
        let mut m = new_tiny_machine();
        close_enough(m.speed_mhz(), 1.0);
        m.cycles = 1_000_000;
        m.timer_ns = 1_000_000_000;
        close_enough(m.speed_mhz(), 1.0);
    }

    #[test]
    fn machine_traps_pc_loop_with_exception() {
        let mut m = new_tiny_machine();
        let mut mem = Memory::new(1024);
        m.run_program(
            &mut mem,
            &[
                0x04, 0x00, 0x00, // 0x0000 JMP $0000
                0x00, //             0x0003 BRK
            ],
        );
        assert_eq!(m.pc, 0x0000, "wrong PC");
    }

    #[test]
    fn load_bin_fn_loads_bin_file() {
        let mut m = new_tiny_machine();
        let mut memory = Memory::new(1024);
        m.load_bin(&mut memory, 0, "tests/lda-ff.bin").unwrap();
        m.run(&mut memory);
        assert_eq!(m.reg("AC"), 0xFF, "wrong AC");
    }
}
