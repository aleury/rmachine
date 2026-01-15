use anyhow::Result;
use clap::Parser;
use clap_num::maybe_hex;

use rmachine_core::monitor::Monitor;
use rmachine_mos6502::MOS6502;

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Cli {
    /// Optional binary file to load
    path: Option<String>,
    /// Load/start address
    #[arg(short, long, value_parser=maybe_hex::<u16>, default_value="0x1000")]
    addr: u16,
    #[arg(short, long)]
    debug: bool,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let mut m = MOS6502::new();

    if let Some(path) = args.path {
        m.load_bin(args.addr, path)?;
    }
    let mut mon = Monitor::new(&mut m);
    mon.debug = args.debug;
    mon.run()
}
