use crate::Machine;

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

    pub fn new_with_debug(machine: &'a mut Machine) -> Self {
        Self {
            debug: true,
            machine,
        }
    }

    pub fn run(&mut self) {
        // let mut input = String::new();

        loop {
            if !self.debug {
                self.machine.run();
                break;
            }

            println!("{}", self.machine);

            self.machine.step();

            // print!("> ");
            // stdout().flush()?;
            // let n = stdin().read_line(&mut input)?;
            // if n == 0 {
            //     break;
            // }
            // match input.trim_end() {
            //     "q" => break,
            //     "n" | "" => {
            //         let mut mysys = &mut sys; // Yeah, I got problems
            //         m.execute_next(&mut mysys).or_else(|e| {
            //             print_state(&m);
            //             bail!(e)
            //         })?;
            //     }
            //     "r" => {
            //         m.run(&mut sys).or_else(|e| {
            //             print_state(&m);
            //             bail!(e)
            //         })?;
            //     }
            //     "?" | "h" | "help" => println!("{HELP}"),
            //     cmd => println!("Unknown command '{cmd}' (type '?' for help)"),
            // }
            // input.clear();
        }
    }
}
