use anyhow::{bail, Result};

use std::io::{stdin, stdout, Write};

use rmachine::prelude::*;

fn main() -> Result<()> {
    let mut m = Machine::new();
    m.load_image(test_program());
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

fn test_program() -> Vec<Word> {
    vec![
        Instruction {
            opcode: Opcode::addi,
            rd: Reg::a0,
            rs1: Reg::zero,
            rs2: Reg::zero,
            imm: 1,
        }
        .into(),
        Instruction {
            opcode: Opcode::auipc,
            rd: Reg::a1,
            rs1: Reg::zero,
            rs2: Reg::zero,
            imm: 0,
        }
        .into(),
        Instruction {
            opcode: Opcode::addi,
            rd: Reg::a1,
            rs1: Reg::a1,
            rs2: Reg::zero,
            imm: 5,
        }
        .into(),
        Instruction {
            opcode: Opcode::addi,
            rd: Reg::a2,
            rs1: Reg::zero,
            rs2: Reg::zero,
            imm: 13,
        }
        .into(),
        Instruction {
            opcode: Opcode::addi,
            rd: Reg::a7,
            rs1: Reg::zero,
            rs2: Reg::zero,
            imm: 64,
        }
        .into(),
        Instruction {
            opcode: Opcode::ecall,
            rd: Reg::zero,
            rs1: Reg::zero,
            rs2: Reg::zero,
            imm: 0,
        }
        .into(),
    ]
}
