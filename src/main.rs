use anyhow::{bail, Result};
use clap::Parser;
use std::io::{stdin, stdout, Write};

use rmachine::prelude::*;

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Cli {
    #[arg(required = true)]
    path: String,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let path = args.path;
    let bytes = std::fs::read(path)?;

    let mut m = Machine::new();
    m.load_image_from_bytes(&bytes);
    let mut input = String::new();
    loop {
        print_state(&m)?;
        print!("> ");
        stdout().flush()?;
        let n = stdin().read_line(&mut input)?;
        if n == 0 {
            break;
        }
        match input.trim_end() {
            "q" => break,
            "n" | "" => {
                m.execute_next().or_else(|e| {
                    print_state(&m)?;
                    bail!(e)
                })?;
            }
            "r" => {
                m.run().or_else(|e| {
                    print_state(&m)?;
                    bail!(e)
                })?;
            }
            "?" | "h" | "help" => println!("{HELP}"),
            cmd => println!("Unknown command '{cmd}' (type '?' for help)"),
        }
        input.clear();
    }
    Ok(())
}

const HELP: &str = "Commands:
Enter - execute next instruction
n - execute next instruction
q - quit
r - run to next breakpoint
? - help";

fn print_state(m: &Machine) -> Result<()> {
    println!(
        "{:4} {:4} {:4} {:4} {:4} {:4} {:4} {:4} {:4}",
        "pc", "a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7"
    );
    println!(
        "{:04x} {:04x} {:04x} {:04x} {:04x} {:04x} {:04x} {:04x} {:04x} : {}",
        m.pc,
        m.regs.get(Reg::a0),
        m.regs.get(Reg::a1),
        m.regs.get(Reg::a2),
        m.regs.get(Reg::a3),
        m.regs.get(Reg::a4),
        m.regs.get(Reg::a5),
        m.regs.get(Reg::a6),
        m.regs.get(Reg::a7),
        Instruction::try_from(m.mem.get(m.pc))?
    );
    Ok(())
}
