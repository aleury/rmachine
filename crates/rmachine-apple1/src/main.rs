use rmachine_core::Memory;
use rmachine_mos6502::{MEMORY_SIZE, MOS6502};

pub trait Component {
    fn step(&mut self, memory: &mut Memory);
}

pub struct System {
    pub cpu: MOS6502,
    pub memory: Memory,
    pub components: Vec<Box<dyn Component>>,
}

impl System {
    pub fn step(&mut self) {
        self.cpu.step(&mut self.memory);
        for component in &mut self.components {
            component.step(&mut self.memory);
        }
    }
}

struct Pia;

impl Component for Pia {
    fn step(&mut self, memory: &mut Memory) {
        let value = memory.get8(0xD012);
        if value & 0b1000_0000 != 0 {
            print!("{}", char::from(value & 0b0111_1111));
            memory.set8(0xD012, value & 0b0111_1111);
        }
    }
}

fn main() {
    let mut system = System {
        memory: Memory::new(MEMORY_SIZE),
        cpu: MOS6502::new(),
        components: vec![Box::new(Pia)],
    };

    system
        .memory
        .load(
            0x0000,
            &[
                0xA9, 193, // 0x0000 LDA 193
                0x2C, 0x12, 0xD0, // 0x0002 ECHO:  BIT $D012
                0x30, 0xFA, // 0x0005 BMI ECHO
                0x8D, 0x12, 0xD0, // 0x0007 STA $D012
                0x69, 0x01, // 0x000A ADC 1
                0x4C, 0x02, 0x00, // 0x000B JMP ECHO
                0x00, // 0x000E BRK
            ],
        )
        .expect("load program into memory");

    for _ in 0..130 {
        system.step();
    }
}
