use anyhow::Result;
use clap::Parser;
use rmachine::prelude::{Instruction, Word};

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Cli {
    #[arg(required = true)]
    path: String,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let bytes = std::fs::read(args.path)?;
    for chunk in bytes[4..].chunks(4) {
        let word = Word::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        println!("{}", Instruction::from(word));
    }

    Ok(())
}
