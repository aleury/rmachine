use std::ops::{Deref, DerefMut};

use rmachine_core::prelude::*;

const REGISTERS: &[&str] = &["SR", "AC", "XR", "YR", "SP"];

const INSTRUCTIONS: &[Instruction] = &[
    Instruction {
        mnemonic: "LDA",
        opcode: 0xA9,
        operation: Operation::LoadImm,
        register: "AC",
    },
    Instruction {
        mnemonic: "INY",
        opcode: 0xC8,
        operation: Operation::IncrementRegister,
        register: "YR",
    },
    Instruction {
        mnemonic: "INX",
        opcode: 0xE8,
        operation: Operation::IncrementRegister,
        register: "XR",
    },
];

#[derive(Debug)]
pub struct MOS6502(Machine);

impl MOS6502 {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deref for MOS6502 {
    type Target = Machine;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MOS6502 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for MOS6502 {
    fn default() -> Self {
        let machine = MachineBuilder::default()
            .with_memory(1024)
            .with_registers(REGISTERS)
            .with_instructions(INSTRUCTIONS)
            .build();
        Self(machine)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn machine_6502_increments_registers_x_and_y() {
        let mut machine = MOS6502::new();
        machine.load(0, &[0xE8, 0xC8]).unwrap();
        machine.step().unwrap();
        assert_eq!(machine.reg("XR"), 1, "wrong X value");
        machine.step().unwrap();
        assert_eq!(machine.reg("YR"), 1, "wrong Y value");
    }

    #[test]
    fn machine_6502_loads_immediate_into_register_a() {
        let mut machine = MOS6502::new();
        machine.load(0, &[0xA9, 0xFF]).unwrap();
        machine.step().unwrap();
        assert_eq!(machine.reg("AC"), 0xFF, "wrong AC value");
    }
}
