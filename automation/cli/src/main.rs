use clap::{Arg, Command};

use automation_lib::{CliError, OutputType, TargetEnv};
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
                        .value_parser(["local", "prod"]),
                )
                .arg(
                    Arg::new("meta_output_type")
                        .short('m')
                        .long("meta_output_type")
                        .required(false)
                        .default_value("json")
                        .value_parser(["bits", "json"]),
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
        Some(("convert_ttf_to_icon", sub_matches)) => {
            let ttf_file_name_val = sub_matches.get_one::<String>("ttf").unwrap();
            automation_lib::convert_ttf_to_icon(ttf_file_name_val)
        }
        Some(("process_assets", sub_matches)) => {
            let mo_val = sub_matches.get_one::<String>("meta_output_type").unwrap();
            let mo_val = match mo_val.as_str() {
                "json" => OutputType::Json,
                "bits" => OutputType::Bits,
                _ => OutputType::Json,
            };

            let env_val = sub_matches.get_one::<String>("env").unwrap();
            match env_val.as_str() {
                "local" => automation_lib::process_assets(env_val, TargetEnv::Local, mo_val),
                "prod" => automation_lib::process_assets(env_val, TargetEnv::Prod, mo_val),
                _ => Err(CliError::Message("Invalid environment".to_string())),
            }
        }
        _ => Err(CliError::Message("Invalid subcommand".to_string())),
    };

    match result {
        Ok(()) => {}
        Err(e) => warn!("Error: {:?}", e),
    }
}
