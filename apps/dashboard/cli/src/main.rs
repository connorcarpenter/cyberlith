use clap::App;
use log::{info, LevelFilter};
use simple_logger::SimpleLogger;

fn main() {

    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .expect("A logger was already initialized");

    let matches = App::new("dashboard_cli")
        .version("0.1.0")
        .author("Connor Carpenter")
        .subcommand(
            App::new("up")
        )
        .subcommand(
            App::new("down")
        )
        .get_matches();

    match matches.subcommand_name() {
        Some("up") => {
            dashboard_lib::up();
        }
        Some("down") => {
            dashboard_lib::down();
        }
        _ => {
            // Handle unknown commands or no command provided
            info!("Invalid command or no command provided");
        }
    }
}