use anyhow::Result;
use clap::Parser;

use rmachine_core::monitor::Monitor;
use rmachine_mos6502::MOS6502;

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

    let program = [
        0xFF, //             0x0000 CLC
        0xD8, //             0x0001 CLD
        0xAD, 0x15, 0x00, // 0x0002 LDA $0015 (adr1)
        0x6D, 0x17, 0x00, // 0x0005 ADC $0017 (adr2)
        0x8D, 0x19, 0x00, // 0x0008 STA $0019 (adr3)
        0xAD, 0x16, 0x00, // 0x000B LDA $0016 (adr1 + 1)
        0x6D, 0x18, 0x00, // 0x000E ADC $0018 (adr2 + 1)
        0x8D, 0x1A, 0x00, // 0x0011 STA $001A (adr3 + 1)
        0x00, //             0x0014 BRK
        0x01, //             0x0015 DB #01
        0x00, //             0x0016 DB #00
        0x34, //             0x0017 DB #34
        0x12, //             0x0018 DB #12
              //             result: 0x0001 + 0x1234 => 0x1235
    ];
    m.load(0, &program).expect("load program");

    let mut mon = Monitor::new(&mut m);
    mon.debug = args.debug;
    mon.run()
}
