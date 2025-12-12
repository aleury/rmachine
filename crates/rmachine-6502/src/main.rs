use rmachine_6502::MOS6502;
use rmachine_core::monitor::Monitor;

fn main() {
    let mut machine = MOS6502::new();
    machine.load(0, &[0xA9, 0xFF]).unwrap();
    let mut mon = Monitor::new_with_debug(&mut machine);
    mon.run();
}
