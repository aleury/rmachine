#![allow(clippy::cast_possible_truncation)]
use std::collections::HashMap;
use std::fmt::Display;
use std::fmt::Write;
use std::thread::sleep;
use std::time::Duration;

use anyhow::Result;
use anyhow::anyhow;

#[derive(Debug, Clone)]
pub enum Mode {
    Implied,
    Immediate,
    Absolute,
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub mnemonic: &'static str,
    pub mode: Mode,
    pub opcode: u8,
    pub bytes: u8,
    pub cycles: u8,
    pub execute: fn(&mut Machine),
}

#[derive(Debug)]
pub struct Machine {
    memory: Vec<u8>,
    pc: u16,
    pub registers: HashMap<&'static str, u8>,
    register_list: &'static [&'static str],
    instructions: HashMap<u8, &'static Instruction>,
    cycles: u32,
    cycle_time_ns: u32,
    timer_ns: usize,
}

#[derive(Debug)]
pub struct MachineBuilder {
    pub memory_size: usize,
    pub registers: &'static [&'static str],
    pub instructions: &'static [Instruction],
    pub frequency_mhz: f32,
}

impl MachineBuilder {
    /// Returns a `Machine` configured according to the builder specification.
    ///
    /// # Panics
    ///
    /// Will panic if an instruction with the same opcode already exists.
    #[must_use]
    pub fn build(self) -> Machine {
        assert!(self.frequency_mhz.is_sign_positive());
        Machine {
            memory: vec![0; self.memory_size],
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
            #[expect(clippy::cast_sign_loss, reason = "previously asserted positive")]
            cycle_time_ns: (1000.0 / self.frequency_mhz) as u32,
            pc: 0,
            cycles: 0,
            timer_ns: 0,
        }
    }
}

impl Machine {
    pub fn run(&mut self) {
        loop {
            if self.step().is_err() {
                break;
            }
        }
    }

    /// # Errors
    ///
    /// May return an error if the execution fails.
    pub fn step(&mut self) -> Result<()> {
        let opcode = self.fetch();

        let instruction = self
            .instructions
            .get(&opcode)
            .copied()
            .ok_or_else(|| anyhow!("opcode not found: {opcode}"))?;

        (instruction.execute)(self);
        self.wait_cycles(instruction.cycles);
        Ok(())
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
    pub fn reg(&mut self, reg_name: &str) -> u8 {
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
    pub fn reg_set(&mut self, reg_name: &'static str, value: u8) {
        self.registers
            .insert(reg_name, value)
            .ok_or_else(|| format!("undefined register '{reg_name}'"))
            .unwrap();
    }

    /// Fetch the next byte from memory.
    ///
    /// # Errors
    ///
    /// Returns an error if the address is out of bounds.
    pub fn fetch(&mut self) -> u8 {
        let value = self.get8(self.pc);
        self.pc += 1;
        value
    }

    /// Reads the byte at the given address from memory.
    #[must_use]
    pub fn get8(&self, addr: u16) -> u8 {
        self.memory.get(addr as usize).copied().unwrap_or(0)
    }

    /// Reads the word at the given address from memory.
    #[must_use]
    pub fn get16(&self, addr: u16) -> u16 {
        let le_bytes = [self.get8(addr), self.get8(addr + 1)];
        u16::from_le_bytes(le_bytes)
    }

    /// Writes a byte to memory at the given address.
    pub fn set8(&mut self, addr: u16, value: u8) {
        if let Some(byte) = self.memory.get_mut(addr as usize) {
            *byte = value;
        }
    }

    /// Load bytes into memory at the given address.
    ///
    /// # Errors
    ///
    /// Returns an error if the address is out of bounds.
    pub fn load(&mut self, addr: usize, program: &[u8]) -> Result<()> {
        let slice = self
            .memory
            .get_mut(addr..addr + program.len())
            .ok_or_else(|| anyhow!("memory out of bounds: {addr:#x}"))?;
        slice.copy_from_slice(program);
        Ok(())
    }

    /// Sleeps long enough to slow the emulator down to roughly the machine's
    /// rated clock frequency.
    ///
    /// Also updates the cycle counter and cycle timer, used to report the
    /// actual speed achieved (by [`Self::speed_mhz`]).
    pub fn wait_cycles(&mut self, cycles: u8) {
        let cycles = u32::from(cycles);
        let delay = cycles * self.cycle_time_ns;
        sleep(Duration::from_nanos(u64::from(delay)));
        let (new_cycles, overflow) = self.cycles.overflowing_add(cycles);
        if overflow {
            self.cycles = 0;
            self.timer_ns = 0;
        } else {
            self.cycles = new_cycles;
            self.timer_ns = self.timer_ns.saturating_add(delay as usize);
        }
    }

    /// Reports the approximate speed achieved by the machine in MHz.
    ///
    /// This is based on the number of cycles executed since the last cycle
    /// counter reset, divided by the internal timer value.
    ///
    /// Emulator overhead is not accounted for, and is assumed to be negligible
    /// relative to the rated clock frequency.
    #[must_use]
    #[expect(clippy::cast_precision_loss, reason = "approximate speed is fine")]
    pub fn speed_mhz(&self) -> f32 {
        if self.cycles == 0 {
            1000.0 / self.cycle_time_ns as f32
        } else {
            self.cycles as f32 / self.timer_ns as f32 * 1000.0
        }
    }

    /// Disassemble the next instruction.
    ///
    /// # Errors
    ///
    /// May return an error if unable to disassemble next instruction.
    #[must_use]
    pub fn disassemble_next(&self) -> Option<String> {
        let opcode = self.memory.get(self.pc as usize)?;
        let instruction = self.instructions.get(opcode)?;
        let mut disassembly = String::from(instruction.mnemonic);
        match instruction.bytes {
            1 => {}
            2 => {
                let value = self.get8(self.pc.wrapping_add(1));
                write!(disassembly, " {value:#04x}").ok()?;
            }
            3 => {
                let value = self.get16(self.pc.wrapping_add(1));
                write!(disassembly, " {value:#06x}").ok()?;
            }
            x => unreachable!("invalid number of bytes: {x}"),
        }
        Some(disassembly)
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
        let instruction = self.disassemble_next().unwrap_or("???".into());
        writeln!(f, "{instruction:10} {:.2}MHz", self.speed_mhz())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_tiny_machine() -> Machine {
        MachineBuilder {
            memory_size: 1024,
            registers: &["A", "X", "Y"],
            instructions: &[
                Instruction {
                    mnemonic: "NOP",
                    mode: Mode::Implied,
                    opcode: 0x00,
                    bytes: 1,
                    cycles: 2,
                    execute: |_| (),
                },
                Instruction {
                    mnemonic: "LDA",
                    mode: Mode::Immediate,
                    opcode: 0x01,
                    bytes: 2,
                    cycles: 2,
                    execute: |m| {
                        let value = m.fetch();
                        m.reg_set("AC", value);
                    },
                },
            ],
            frequency_mhz: 1.0,
        }
        .build()
    }

    #[test]
    fn machine_new_returns_initialized_machine() {
        let mut machine = new_tiny_machine();

        machine.load(0, &[0x00]).unwrap();

        machine.step().unwrap();

        assert_eq!(machine.pc, 1);
    }

    #[test]
    fn cycles_are_counted() {
        let mut machine = new_tiny_machine();
        machine
            .load(
                0,
                &[
                    0x00, // 0x0000 NOP (2 cycles)
                    0x00, // 0x0001 NOP (2 cycles)
                ],
            )
            .unwrap();
        machine.step().unwrap();
        machine.step().unwrap();
        assert_eq!(machine.cycles, 4);
    }

    #[test]
    fn get8_returns_a_byte_from_memory() {
        let mut machine = new_tiny_machine();

        machine.load(0, &[0xFF]).unwrap();

        assert_eq!(machine.get8(0), 0xFF);
    }

    #[test]
    fn get16_returns_a_16_bit_value_from_memory() {
        let mut machine = new_tiny_machine();

        machine.load(0, &[0xEF, 0xBE]).unwrap();

        assert_eq!(machine.get16(0), 0xBEEF);
    }

    #[test]
    fn set8_writes_a_byte_to_memory() {
        let mut machine = new_tiny_machine();

        machine.set8(0, 0x42);

        assert_eq!(machine.get8(0), 0x42);
    }

    #[test]
    fn set8_ignores_out_of_bounds_writes() {
        let mut machine = new_tiny_machine();

        // Memory size is 1024 bytes (0-1023), so address 2000 is out of bounds
        machine.set8(2000, 0xFF);

        // Should not panic, and reading out of bounds returns 0
        assert_eq!(machine.get8(2000), 0x00);
    }

    // #[test]
    // fn load_loads_bytes_into_memory_at_the_given_address() {
    //     let mut machine = Machine::<TinyISA>::new(4);
    //     machine.load(0, &[0x01, 42]).unwrap();

    //     assert_eq!(machine.memory, Memory(vec![0x01, 42, 0, 0]));
    // }

    // #[test]
    // fn step_increments_pc_register_after_executing_nop_instruction() {
    //     let mut machine = Machine::<TinyISA>::new(2);
    //     machine.load(0, &[0x00]).unwrap();

    //     machine.step().unwrap();

    //     assert_eq!(machine.cpu.pc, 1);
    // }

    // #[test]
    // fn run_executes_instructions_until_an_error_occurs() {
    //     let mut machine = Machine::<TinyISA>::new(256);
    //     machine.load(0, &[0x01, 42, 0xFF]).unwrap(); // lda immediate

    //     let result = machine.run();

    //     assert!(result.is_err());
    //     assert_eq!(machine.cpu.a, 42);
    //     assert_eq!(machine.cpu.pc, 3);
    // }
}
