use clap::{crate_version, App, AppSettings};

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
        .args_from_usage("<INPUT_FILE>         'Your RISC-V file'")
        .args_from_usage("[PORT] -p, --port    'MIDI output port'");

    app
}
