use std::io::{Write, stdin, stdout};

use crate::Machine;
use anyhow::Result;

pub struct Monitor<'a> {
    pub debug: bool,
    machine: &'a mut Machine,
}

impl<'a> Monitor<'a> {
    pub fn new(machine: &'a mut Machine) -> Self {
        Self {
            debug: false,
            machine,
        }
    }

    /// Run monitor until the machine halts.
    ///
    /// # Errors
    ///
    /// This function will return an error if the machine encounters an error.
    ///
    /// # Panics
    ///
    /// If the memory dump hits an out-of-range address.
    pub fn run(&mut self) -> Result<()> {
        let mut input = String::new();

        loop {
            if !self.debug {
                self.machine.run();
            }

            println!("{}", self.machine);
            print!("> ");
            stdout().flush()?;
            let n = stdin().read_line(&mut input)?;
            if n == 0 {
                break;
            }
            let mut tokens = input.split_whitespace();
            match tokens.next() {
                Some("p") => {
                    self.debug = true;
                    if let Some(addr) = tokens.next() {
                        match u16::from_str_radix(addr, 16) {
                            Ok(addr) => self.machine.pc = addr,
                            Err(_) => eprintln!("Invalid address: {addr}"),
                        }
                    } else {
                        eprintln!("Usage: p <address>");
                    }
                }
                Some("q") => break,
                Some("n") | None => self.machine.step(),
                Some("m") => {
                    let page_start = self.machine.pc & 0xFF00;
                    for row in 0..16_u16 {
                        let row_offset = row.checked_mul(16).expect("address out of range");
                        let row_start = page_start
                            .checked_add(row_offset)
                            .expect("address out of range");
                        print!("{row_start:04X}:");
                        for col in 0..16 {
                            let addr = row_start.checked_add(col).expect("address out of range");
                            print!(" {:02X}", self.machine.get8(addr));
                        }
                        print!("  |");
                        for col in 0..16 {
                            let addr = row_start.checked_add(col).expect("address out of range");
                            let byte = self.machine.get8(addr);
                            print!(
                                "{}",
                                if byte.is_ascii_graphic() || byte == b' ' {
                                    char::from(byte)
                                } else {
                                    '.'
                                }
                            );
                        }
                        println!("|");
                    }
                }
                Some("r") => self.machine.run(),
                Some("?" | "h" | "help") => println!("{HELP}\n"),
                Some(cmd) => println!("Unknown command '{cmd}' (type '?' for help)"),
            }
            input.clear();
        }

        Ok(())
    }
}

const HELP: &str = "Commands:
Enter - execute next instruction
m - dump current 256-byte page of memory
n - execute next instruction
p ADDR - set PC to ADDR (hex)
q - quit
r - run to next breakpoint
? - help";
