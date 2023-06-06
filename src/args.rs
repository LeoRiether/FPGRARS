use clap::Parser;

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
    Args::parse()
}
