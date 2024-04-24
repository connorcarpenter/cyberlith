use clap::{Arg, Command};

use automation_lib::CliError;
use logging::warn;

fn cli() -> Command {
    Command::new("deploy_cli")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(Command::new("up").about("starts live services"))
        .subcommand(Command::new("up_content").about("updates prod content server"))
        .subcommand(Command::new("down").about("down live services"))
        .subcommand(
            Command::new("process_assets")
                .about("processes assets for a given environment")
                .arg_required_else_help(true)
                .arg(
                    Arg::new("env")
                        .short('e')
                        .long("env")
                        .required(true)
                        .value_parser(["dev", "stage", "prod", "qwon"]),
                ),
        )
        .subcommand(
            Command::new("convert_ttf_to_icon")
                .about("converts ttf to icon")
                .arg_required_else_help(true)
                .arg(Arg::new("ttf").short('t').long("ttf").required(true)),
        )
}

fn main() {
    logging::initialize();

    let matches = cli().get_matches();

    let result = match matches.subcommand() {
        Some(("up", _sub_matches)) => automation_lib::up(),
        Some(("down", _sub_matches)) => automation_lib::down(),
        Some(("process_assets", sub_matches)) => {
            let env_val = sub_matches.get_one::<String>("env").unwrap();
            automation_lib::process_assets(env_val)
        }
        Some(("convert_ttf_to_icon", sub_matches)) => {
            let ttf_file_name_val = sub_matches.get_one::<String>("ttf").unwrap();
            automation_lib::convert_ttf_to_icon(ttf_file_name_val)
        }
        _ => Err(CliError::Message("Invalid subcommand".to_string())),
    };

    match result {
        Ok(()) => {}
        Err(e) => warn!("Error: {:?}", e),
    }
}
