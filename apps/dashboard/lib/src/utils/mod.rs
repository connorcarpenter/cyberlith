use log::info;
use openssh::Session;
use vultr::VultrError;

pub async fn run_ssh_command(session: &Session, command_str: &str) -> Result<(), VultrError> {
    info!("-> {}", command_str);

    let commands: Vec<String> = command_str.split(" ").map(|thestr| thestr.to_string()).collect();

    let mut command = session.command(&commands[0]);
    for i in 1..commands.len() {
        command.arg(&commands[i]);
    }

    let output = command.output().await.map_err(|err| VultrError::Dashboard(err.to_string()))?;
    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout);
        info!("<- {}", result);
        return Ok(());
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr);
        return Err(VultrError::Dashboard(format!("Command Error: {}", error_message)));
    }
}

pub async fn run_ssh_raw_command(session: &Session, command_str: &str) -> Result<(), VultrError> {
    info!("-> {}", command_str);

    let mut raw_command = session.raw_command(command_str);
    let output = raw_command.output().await.map_err(|err| VultrError::Dashboard(err.to_string()))?;
    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout);
        info!("<- {}", result);
        return Ok(());
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr);
        return Err(VultrError::Dashboard(format!("Command Error: {}", error_message)));
    }
}