use clap::Parser;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    #[arg(short, long)]
    pub port: Option<usize>,

    #[arg(long)]
    pub no_video: bool,

    /// Prints the instructions in the FPGRARS format
    #[arg(long)]
    pub print_instructions: bool,

    /// Prints the final state of the program after execution
    #[arg(long)]
    pub print_state: bool,

    /// The RISC-V file to execute
    pub file: String,
}

pub fn get_args() -> Args {
    let args = Args::parse();

    if !Path::new(&args.file).is_file() {
        panic!("<{}> must be a valid file", args.file);
    }

    args
}
