use std::future::Future;

use async_compat::Compat;
use crossbeam_channel::{bounded, Receiver};
use log::{info, warn};
use openssh::Session;
use subprocess::{Exec, Redirection};
use vultr::VultrError;

pub fn thread_init<F: Future<Output=Result<(), VultrError>> + Sized + Send + 'static>(
    x: fn() -> F
) -> Receiver<Result<(), VultrError>> {
    let (sender, receiver) = bounded(1);

    executor::spawn(async move {
        let result = x().await;
        sender.send(result).expect("failed to send result");
    })
        .detach();

    receiver
}

pub fn thread_init_compat<F: Future<Output=Result<(), VultrError>> + Sized + Send + 'static>(
    x: fn() -> F
) -> Receiver<Result<(), VultrError>> {
    let (sender, receiver) = bounded(1);

    executor::spawn(Compat::new(async move {
        let result = x().await;
        sender.send(result).expect("failed to send result");
    }))
        .detach();

    receiver
}

pub async fn run_command(command_str: &str) -> Result<(), VultrError> {

    info!("(local) -> {}", command_str);

    let command = Exec::shell(command_str)
        .stdout(Redirection::Pipe)
        .stderr(Redirection::Pipe);
    let data_capture = match command.capture() {
        Ok(data_capture) => data_capture,
        Err(err) => {
            warn!("(local error) <- {}", err);
            return Err(VultrError::Dashboard(format!("Command Error: {}", err)));
        }
    };
    if data_capture.success() {
        let out = data_capture.stdout_str().trim().to_string();
        info!("(local) <- {}", out);
        return Ok(());
    } else {
        let err = data_capture.stderr_str().trim().to_string();
        warn!("(local error) <- {}", err);
        return Err(VultrError::Dashboard(format!("Command Error: {}", err)));
    }
}

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