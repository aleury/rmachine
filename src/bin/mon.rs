use anyhow::{bail, Result};
use clap::Parser;
use std::io::{stdin, stdout, Write};

use rmachine::{prelude::*, try_image_from_bytes};

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Cli {
    path: Option<String>,
    #[arg(short, long)]
    debug: bool,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let mut m = Machine::default();
    let mut sys = TermSys::new();

    let mut debug = args.debug;
    if let Some(path) = args.path {
        let bytes = std::fs::read(path)?;
        let image = try_image_from_bytes(&bytes)?;
        m.load_image(&image);
    } else {
        debug = true;
    }

    let mut input = String::new();

    loop {
        if !debug {
            if let Ok(()) = m.run(&mut sys) {
                break;
            }
        }

        print_state(&m);
        print!("> ");
        stdout().flush()?;
        let n = stdin().read_line(&mut input)?;
        if n == 0 {
            break;
        }
        match input.trim_end() {
            "q" => break,
            "n" | "" => {
                let mut mysys = &mut sys; // Yeah, I got problems
                m.execute_next(&mut mysys).or_else(|e| {
                    print_state(&m);
                    bail!(e)
                })?;
            }
            "r" => {
                m.run(&mut sys).or_else(|e| {
                    print_state(&m);
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

fn print_state(m: &Machine) {
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
        Instruction::from(m.mem.get(m.pc)),
    );
}
