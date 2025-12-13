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

    let mut machine = MOS6502::new();
    machine.load(0, &[0xA9, 0xFF]).expect("load program");

    let mut mon = Monitor::new(&mut machine);
    mon.debug = args.debug;
    mon.run()
}
