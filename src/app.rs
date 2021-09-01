use clap::{crate_version, App, AppSettings};
use std::path::Path;

macro_rules! exit {
    ($($arg:tt)*) => {{
        eprintln!($($arg)*);
        std::process::exit(1)
    }}
}

pub struct Args {
    pub file: String,
    pub port: Option<usize>,
    pub video: bool,
}

pub fn build_app() -> App<'static, 'static> {
    let clap_color_setting = if std::env::var_os("NO_COLOR").is_none() {
        AppSettings::ColoredHelp
    } else {
        AppSettings::ColorNever
    };

    let app = App::new("fpgrars")
        .version(crate_version!())
        .setting(clap_color_setting)
        .about("A RISC-V simulator with built-in graphics display and keyboard input")
        .arg_from_usage("<input_file>      'Your RISC-V file'")
        .arg_from_usage("-p, --port=[PORT] 'MIDI output port (defaults to 0)'")
        .arg_from_usage("--no-video        'Don't show the bitmap display'");

    app
}

pub fn get_args() -> Args {
    let matches = build_app().get_matches();

    let file = match matches.value_of("input_file") {
        None => exit!("Failed to get <INPUT_FILE> argument"),
        Some(file) if !Path::new(file).is_file() => exit!("`{}` must be a valid file", file),
        Some(file) => file.to_string(),
    };

    let port = match matches.value_of("port").map(|x| (x, x.parse::<usize>().ok())) {
        None => None,
        Some((x, None)) => exit!("Invalid port number `{}`", x), // parse failed
        Some((_, Some(p))) => Some(p),
    };

    let video = !matches.is_present("no-video");

    Args {
        file,
        port,
        video,
    }
}
