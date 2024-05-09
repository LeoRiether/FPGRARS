use clap::Parser;
use serde::Deserialize;

#[derive(Parser, Deserialize, Debug, Default)]
#[command(author, version, about)]
#[clap(disable_help_flag = true)]
pub struct OptionalConfig {
    #[clap(long, action = clap::ArgAction::HelpLong)]
    help: Option<bool>,

    /// Hides the bitmap display
    #[arg(long)]
    pub no_video: bool,

    /// The width of the bitmap display. Defaults to 320px
    #[arg(short, long)]
    pub width: Option<usize>,

    /// The height of the bitmap display. Defaults to 240px
    #[arg(short, long)]
    pub height: Option<usize>,

    /// Each pixel is scaled by this factor. Defaults to 2 (each pixel becomes a 2x2 square)
    #[arg(short, long)]
    pub scale: Option<usize>,

    /// The MIDI port to use for audio
    #[arg(short, long)]
    pub port: Option<usize>,

    /// Prints the instructions in the FPGRARS format
    #[arg(long)]
    pub print_instructions: bool,

    /// Prints the final state of the program after execution
    #[arg(long)]
    pub print_state: bool,

    /// The RISC-V file to execute
    pub file: Option<String>,
}

impl OptionalConfig {
    pub fn get_args() -> Self {
        Self::parse()
    }

    pub fn get_toml() -> Self {
        std::fs::read_to_string("fpgrars.toml")
            .ok()
            .map(|config| toml::from_str(&config).expect("Failed to parse config file"))
            .unwrap_or_default()
    }

    pub fn merge(self, rhs: Self) -> Self {
        Self {
            help: self.help.or(rhs.help),
            no_video: self.no_video || rhs.no_video,
            width: self.width.or(rhs.width),
            height: self.height.or(rhs.height),
            scale: self.scale.or(rhs.scale),
            port: self.port.or(rhs.port),
            print_instructions: self.print_instructions || rhs.print_instructions,
            print_state: self.print_state || rhs.print_state,
            file: self.file.or(rhs.file),
        }
    }
}

#[derive(Debug, Default)]
pub struct Config {
    pub no_video: bool,
    pub width: usize,
    pub height: usize,
    pub scale: usize,
    pub port: Option<usize>,
    pub print_instructions: bool,
    pub print_state: bool,
    pub file: String,
}

impl From<OptionalConfig> for Config {
    fn from(config: OptionalConfig) -> Self {
        Self {
            no_video: config.no_video,
            width: config.width.unwrap_or(320),
            height: config.height.unwrap_or(240),
            scale: config.scale.unwrap_or(2),
            port: config.port,
            print_instructions: config.print_instructions,
            print_state: config.print_state,
            file: config.file.unwrap_or_else(|| {
                eprintln!("No file specified");
                std::process::exit(1);
            }),
        }
    }
}

impl Config {
    pub fn get() -> Self {
        OptionalConfig::get_toml()
            .merge(OptionalConfig::get_args())
            .into()
    }
}
