use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    #[arg(long)]
    pub no_video: bool,

    #[arg(short, long, default_value = "320")]
    pub width: usize,

    #[arg(short, long, default_value = "240")]
    pub height: usize,

    #[arg(short, long, default_value = "2")]
    pub scale: usize,

    #[arg(short, long)]
    pub port: Option<usize>,

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
