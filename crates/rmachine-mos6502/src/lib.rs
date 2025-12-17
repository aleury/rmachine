use std::ops::{Deref, DerefMut};

use rmachine_core::prelude::*;

const REGISTERS: &[&str] = &["PC", "AC", "X", "Y", "SR", "SP"];

const INSTRUCTIONS: &[Instruction] = &[
    Instruction {
        mnemonic: "lda",
        opcode: 0xA9,
        operation: Operation::LoadImm,
        register: "AC",
    },
    Instruction {
        mnemonic: "inx",
        opcode: 0xE8,
        operation: Operation::IncrementRegister,
        register: "X",
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
    fn machine_6502_increments_register_x() {
        let mut machine = MOS6502::new();

        machine.load(0, &[0xE8]).unwrap();

        machine.step().unwrap();

        let want = 1;
        let got = machine.registers.get("X").copied().unwrap();
        assert_eq!(want, got);
    }

    #[test]
    fn machine_6502_loads_immediate_into_register_a() {
        let mut machine = MOS6502::new();

        machine.load(0, &[0xA9, 0xFF]).unwrap();

        machine.step().unwrap();

        let want = 0xFF;
        let got = machine.registers.get("AC").copied().unwrap();
        assert_eq!(want, got);
    }
}
