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
            match input.trim_end() {
                "q" => break,
                "n" | "" => self.machine.step(),
                "m" => {
                    for row in 0..8_u16 {
                        let base: u16 = row.checked_mul(16).expect("address out of range");
                        print!("{base:04X}:");
                        for col in 0..16 {
                            let addr = base.checked_add(col).expect("address out of range");
                            print!(" {:02X}", self.machine.get8(addr));
                        }
                        print!("  |");
                        for col in 0..16 {
                            let addr = base.checked_add(col).expect("address out of range");
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
                "r" => self.machine.run(),
                "?" | "h" | "help" => println!("{HELP}\n"),
                cmd => println!("Unknown command '{cmd}' (type '?' for help)"),
            }
            input.clear();
        }

        Ok(())
    }
}

const HELP: &str = "Commands:
Enter - execute next instruction
m - dump first 128 bytes of memory
n - execute next instruction
q - quit
r - run to next breakpoint
? - help";
