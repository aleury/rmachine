use anyhow::Result;
use clap::Parser;
use rmachine_legacy::prelude::Instruction;
use rmachine_legacy::try_image_from_bytes;

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Cli {
    #[arg(required = true)]
    path: String,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let bytes = std::fs::read(args.path)?;
    let image = try_image_from_bytes(&bytes)?;
    for word in image {
        println!("{}", Instruction::from(word));
    }
    Ok(())
}
