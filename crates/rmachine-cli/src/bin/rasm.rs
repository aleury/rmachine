use anyhow::{Result, bail};
use clap::Parser;
use rmachine_legacy::build_exe;

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Cli {
    /// Input assembly files to compile
    #[arg(required = true)]
    input: Vec<String>,

    /// Output file path (only valid with single input file)
    #[arg(short, long)]
    output: Option<String>,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    // Check that -o is only used with a single input file
    if args.output.is_some() && args.input.len() > 1 {
        bail!("Cannot use -o flag with multiple input files");
    }

    for input_file in &args.input {
        let output_path = if let Some(ref output) = args.output {
            // Use the specified output path
            output.as_str()
        } else {
            // Default behavior: strip .s extension
            input_file.strip_suffix(".s").ok_or_else(|| {
                anyhow::anyhow!("Input file must end with .s extension: {input_file}")
            })?
        };

        build_exe(input_file, output_path)?;

        // Print success message
        if args.output.is_some() {
            println!("Assembled {input_file} -> {output_path}");
        } else {
            println!("Assembled {input_file}");
        }
    }

    Ok(())
}
