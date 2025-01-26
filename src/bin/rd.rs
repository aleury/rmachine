use anyhow::{bail, Result};
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

    let mut offset = 0;

    // Read the magic number.
    let header = &bytes[offset..4];
    offset += 4;
    if header != b"rme1" {
        bail!("Invalid magic number");
    }
    println!(
        "{}\n",
        &header.iter().map(|b| *b as char).collect::<String>()
    );

    // Read the text section length.
    let text_len = Word::from_be_bytes(bytes[offset..offset + 4].try_into()?);
    offset += size_of::<Word>();

    // Read the text section.
    println!(".text\n------");
    let text_chunks = bytes[offset..offset + text_len as usize].chunks(4);
    for chunk in text_chunks {
        let word = Word::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        println!("{}", Instruction::from(word));
    }
    offset += text_len as usize;

    // Read the data section length.
    let data_len = Word::from_be_bytes(bytes[offset..offset + 4].try_into()?);
    offset += size_of::<Word>();

    // Read the data section.
    println!("\n.data\n------");
    let data_chunks = bytes[offset..offset + data_len as usize].chunks(4);
    for chunk in data_chunks {
        let word = Word::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        print!("{word} ");
    }
    println!("\n");

    Ok(())
}
