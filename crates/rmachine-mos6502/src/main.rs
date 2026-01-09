use anyhow::Result;
use clap::Parser;

use rmachine_core::monitor::Monitor;
use rmachine_mos6502::MOS6502;

const CARRY: u8 = 0b0000_0001;

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Cli {
    path: Option<String>,
    #[arg(short, long)]
    debug: bool,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let mut m = MOS6502::new();
    // let program = [machine.opcode("LDA"), 0xFF];

    let status = m.reg_mut("SR");
    *status |= CARRY;

    // let program = [m.opcode("LDA"), 0xFF, m.opcode("ADC"), 0x01];
    let program = [
        0xAD, 0x04, 0x00, // 0x0000 LDA $0004
        0x00, //             0x0003 HLT
        0xFF, //             0x0004 DB #FF
    ];
    m.load(0, &program).expect("load program");

    let mut mon = Monitor::new(&mut m);
    mon.debug = args.debug;
    mon.run()
}
