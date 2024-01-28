use clap::Command;
use log::LevelFilter;
use simple_logger::SimpleLogger;

fn cli() -> Command {
    Command::new("dashboard_cli")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("up")
                .about("starts live services"),
        )
        .subcommand(
            Command::new("down")
                .about("down live services"),
        )
}

fn main() {

    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .expect("A logger was already initialized");

    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("up", _sub_matches)) => {
            dashboard_lib::up();
        }
        Some(("down", _sub_matches)) => {
            dashboard_lib::down();
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable!()
    }
}