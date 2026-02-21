use rmachine_core::Memory;
use rmachine_mos6502::MOS6502;

pub struct System {
    pub memory: Memory,
    pub cpu: MOS6502,
}

fn main() {
    println!("hello world!");
}
