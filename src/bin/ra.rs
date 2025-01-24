use anyhow::Result;
use clap::Parser;
use rmachine::build_exe;

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Cli {
    #[arg(required = true)]
    filenames: Vec<String>,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    for filename in args.filenames {
        build_exe(filename.as_str(), filename.strip_suffix(".s").unwrap())?;
    }

    Ok(())
}
