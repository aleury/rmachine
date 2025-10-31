#![allow(clippy::cast_possible_truncation)]
use std::collections::HashMap;

use anyhow::Result;
use anyhow::anyhow;

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Register(&'static str);

impl From<&'static str> for Register {
    fn from(value: &'static str) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone)]
pub enum Operation {
    NoOp,
    LoadImm,
    IncrementRegister,
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub mnemonic: String,
    pub opcode: u8,
    pub operation: Operation,
    pub register: Register,
}

#[derive(Default)]
pub struct Machine {
    memory: Vec<u8>,
    pc: u16,
    registers: HashMap<Register, u8>,
    instructions: HashMap<u8, Instruction>,
}

#[derive(Default)]
pub struct MachineBuilder {
    machine: Machine,
}

impl MachineBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            machine: Machine::default(),
        }
    }

    #[must_use]
    pub fn with_memory(mut self, memory_size: usize) -> Self {
        self.machine.memory.resize(memory_size, 0);
        self
    }

    #[must_use]
    pub fn with_registers(mut self, registers: Vec<&'static str>) -> Self {
        for register in registers {
            self.machine.registers.insert(Register(register), 0);
        }
        self
    }

    #[must_use]
    pub fn with_instructions(mut self, instructions: Vec<Instruction>) -> Self {
        for instruction in instructions {
            self.machine
                .instructions
                .insert(instruction.opcode, instruction);
        }
        self
    }

    #[must_use]
    pub fn build(self) -> Machine {
        self.machine
    }
}

impl Machine {
    /// # Panics
    ///
    /// May panic for unimplemented opcode.
    pub fn step(&mut self) {
        let opcode = self.next();

        let instruction = self.instructions.get(&opcode).cloned().unwrap();

        #[expect(unreachable_patterns, reason = "we're not done yet")]
        match instruction {
            Instruction {
                operation: Operation::NoOp,
                ..
            } => {}
            Instruction {
                operation: Operation::LoadImm,
                register,
                ..
            } => {
                let imm = self.next();
                self.registers
                    .entry(register.clone())
                    .and_modify(|value| *value = imm);
            }
            Instruction {
                operation: Operation::IncrementRegister,
                register,
                ..
            } => {
                self.registers
                    .entry(register.clone())
                    .and_modify(|value| *value += 1);
            }
            instruction => unimplemented!("unknown instruction: {instruction:#?}"),
        }
    }

    fn next(&mut self) -> u8 {
        let value = *self.memory.get(self.pc as usize).unwrap();
        self.pc += 1;
        value
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
}

// #[derive(Debug, Default)]
// pub struct Machine<ISA: InstructionSet> {
//     cpu: ISA::Cpu,
//     memory: Memory,
//     _phantom: PhantomData<ISA>,
// }

// impl<ISA: InstructionSet> Machine<ISA> {
//     #[must_use]
//     /// Create a new machine with the given memory size.
//     pub fn new(memory_size: usize) -> Self {
//         Self {
//             cpu: ISA::Cpu::default(),
//             memory: Memory::new(memory_size),
//             _phantom: PhantomData,
//         }
//     }

//     /// Step the machine by one instruction
//     ///
//     /// # Errors
//     ///
//     /// Returns an error if the instruction is illegal or if there is an error executing the instruction.
//     pub fn step(&mut self) -> Result<()> {
//         self.cpu.step(&mut self.memory)
//     }

//     /// Run the machine until an error occurs.
//     ///
//     /// # Errors
//     ///
//     /// Returns an error when an instruction fails (illegal opcode, memory fault, etc.)
//     pub fn run(&mut self) -> Result<()> {
//         loop {
//             self.step()?;
//         }
//     }

//     /// Load a program into memory at the given address.
//     ///
//     /// # Errors
//     ///
//     /// Returns an error if there is an error writing to memory.
//     pub fn load(&mut self, addr: usize, program: &[u8]) -> Result<()> {
//         self.memory.load(addr, program)
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    fn new_tiny_machine() -> Machine {
        let registers = vec!["a", "x", "y"];
        let instructions = vec![
            Instruction {
                mnemonic: "nop".to_string(),
                opcode: 0x00,
                operation: Operation::NoOp,
                register: Register::default(),
            },
            Instruction {
                mnemonic: "lda".to_string(),
                opcode: 0x01,
                operation: Operation::LoadImm,
                register: Register("a"),
            },
        ];

        MachineBuilder::default()
            .with_memory(1024)
            .with_registers(registers)
            .with_instructions(instructions)
            .build()
    }

    fn new_6502_machine() -> Machine {
        let registers = vec!["a", "x", "y", "f"];
        let instructions = vec![
            Instruction {
                mnemonic: "lda".to_string(),
                opcode: 0xA9,
                operation: Operation::LoadImm,
                register: Register("a"),
            },
            Instruction {
                mnemonic: "inx".to_string(),
                opcode: 0xE8,
                operation: Operation::IncrementRegister,
                register: Register("x"),
            },
        ];
        MachineBuilder::default()
            .with_memory(1024)
            .with_registers(registers)
            .with_instructions(instructions)
            .build()
    }

    #[test]
    fn machine_new_returns_initialized_machine() {
        let mut machine = new_tiny_machine();

        machine.load(0, &[0x00]).unwrap();

        machine.step();

        assert_eq!(machine.pc, 1);
    }

    #[test]
    fn machine_6502_increments_register_x() {
        let mut machine = new_6502_machine();

        machine.load(0, &[0xE8]).unwrap();

        machine.step();

        let want = 1;
        let got = machine.registers.get(&"x".into()).copied().unwrap();
        assert_eq!(want, got);
    }

    #[test]
    fn machine_6502_loads_immediate_into_register_a() {
        let mut machine = new_6502_machine();

        machine.load(0, &[0xA9, 0xFF]).unwrap();

        machine.step();

        let want = 0xFF;
        let got = machine.registers.get(&"a".into()).copied().unwrap();
        assert_eq!(want, got);
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
