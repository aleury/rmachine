use std::ops::{Deref, DerefMut};

use rmachine_core::prelude::*;

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
        let machine = MachineBuilder::default()
            .with_memory(1024)
            .with_registers(registers)
            .with_instructions(instructions)
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
        let got = machine.registers.get(&"x".into()).copied().unwrap();
        assert_eq!(want, got);
    }

    #[test]
    fn machine_6502_loads_immediate_into_register_a() {
        let mut machine = MOS6502::new();

        machine.load(0, &[0xA9, 0xFF]).unwrap();

        machine.step().unwrap();

        let want = 0xFF;
        let got = machine.registers.get(&"a".into()).copied().unwrap();
        assert_eq!(want, got);
    }
}
