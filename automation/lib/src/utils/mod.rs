use std::{future::Future, path::Path};

use async_compat::Compat;
use crossbeam_channel::{bounded, Receiver, TryRecvError};
use logging::{info, warn};
use openssh::{KnownHosts, Session, SessionBuilder};
use smol::channel::bounded as smol_bounded;
use subprocess::{Exec, Redirection};

use crate::CliError;

pub fn thread_init<F: Future<Output = Result<(), CliError>> + Sized + Send + 'static>(
    x: fn() -> F,
) -> Receiver<Result<(), CliError>> {
    let (sender, receiver) = bounded(1);

    executor::spawn(async move {
        let result = x().await;
        sender.send(result).expect("failed to send result");
    })
    .detach();

    receiver
}

pub fn thread_init_compat<F: Future<Output = Result<(), CliError>> + Sized + Send + 'static>(
    x: fn() -> F,
) -> Receiver<Result<(), CliError>> {
    let (sender, receiver) = bounded(1);

    executor::spawn(Compat::new(async move {
        let result = x().await;
        sender.send(result).expect("failed to send result");
    }))
    .detach();

    receiver
}

pub fn thread_init_compat_1arg<
    A: Send + 'static,
    F: Future<Output = Result<(), CliError>> + Sized + Send + 'static,
>(
    a: A,
    x: fn(A) -> F,
) -> Receiver<Result<(), CliError>> {
    let (sender, receiver) = bounded(1);

    executor::spawn(Compat::new(async move {
        let result = x(a).await;
        sender.send(result).expect("failed to send result");
    }))
    .detach();

    receiver
}

pub async fn run_command(command_name: &str, command_str: &str) -> Result<(), CliError> {
    // info!("({}) -> {}", command_name, command_str);

    let command_name = command_name.to_string();
    let command_name_clone = command_name.clone();
    let command_str = command_str.to_string();

    let (sender, receiver) = smol_bounded(1);

    executor::spawn(async move {
        let command_name = command_name_clone;

        let result_to_send = run_command_blocking(&command_name, &command_str);

        sender
            .send(result_to_send)
            .await
            .expect("failed to send result");
    })
    .detach();

    match receiver.recv().await {
        Ok(Ok(())) => {
            // info!("({}) received successful(?) status from command", command_name);
            Ok(())
        }
        Ok(Err(err)) => {
            warn!("({}) error: {:?}", command_name, err);
            Err(CliError::Message(err.to_string()))
        }
        Err(err) => {
            warn!("({}) error: {:?}", command_name, err);
            Err(CliError::Message(err.to_string()))
        }
    }
}

pub fn run_command_blocking(command_name: &str, command_str: &str) -> Result<(), CliError> {
    info!("({}) -> {}", command_name, command_str);

    let command_name = command_name.to_string();
    let command_name_clone = command_name.clone();
    let command_str = command_str.to_string();

    let command_name = command_name_clone;

    let args = command_str
        .split(" ")
        .map(|thestr| thestr.to_string())
        .collect::<Vec<String>>();

    let command = Exec::cmd(&args[0])
        .args(&args[1..args.len()])
        .stdout(Redirection::Pipe)
        .cwd("/home/connor/Work/cyberlith");
    match command.capture() {
        Ok(capture) => {
            let out = capture.stdout_str();
            if out.len() > 0 {
                let lines = out.lines().map(String::from).collect::<Vec<String>>();
                for line in lines {
                    info!("({}) <- {}", command_name, line);
                }
            }
            Ok(())
        }
        Err(err) => Err(CliError::Message(err.to_string())),
    }
}

pub async fn ssh_session_create() -> Result<Session, CliError> {
    info!("preparing to SSH into instance");

    let key_path = Path::new("~/Work/cyberlith/.vultr/vultrkey");

    let ssh_path = "ssh://root@cyberlith.com";

    let session_opt;
    loop {
        let session_result = SessionBuilder::default()
            .known_hosts_check(KnownHosts::Accept)
            .keyfile(key_path)
            .connect(&ssh_path)
            .await;
        match session_result {
            Ok(session) => {
                session_opt = Some(session);
                break;
            }
            Err(err) => {
                warn!("error connecting to instance: {:?}", err);
                warn!("retrying after 5 seconds..");
                smol::Timer::after(std::time::Duration::from_secs(5)).await;
                continue;
            }
        }
    }

    info!("SSH session established");

    Ok(session_opt.unwrap())
}

pub async fn ssh_session_close(session: Session) -> Result<(), CliError> {
    session
        .close()
        .await
        .map_err(|err| CliError::Message(err.to_string()))
}

pub async fn run_ssh_command(session: &Session, command_str: &str) -> Result<(), CliError> {
    info!("-> {}", command_str);

    let commands: Vec<String> = command_str
        .split(" ")
        .map(|thestr| thestr.to_string())
        .collect();

    let mut command = session.command(&commands[0]);
    for i in 1..commands.len() {
        command.arg(&commands[i]);
    }

    let output = command
        .output()
        .await
        .map_err(|err| CliError::Message(err.to_string()))?;
    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout);
        info!("<- {}", result);
        return Ok(());
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr);
        return Err(CliError::Message(format!(
            "Command Error: {}",
            error_message
        )));
    }
}

pub async fn run_ssh_raw_command(session: &Session, command_str: &str) -> Result<(), CliError> {
    info!("-> {}", command_str);

    let mut raw_command = session.raw_command(command_str);
    let output = raw_command
        .output()
        .await
        .map_err(|err| CliError::Message(err.to_string()))?;
    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout);
        info!("<- {}", result);
        return Ok(());
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr);
        return Err(CliError::Message(format!(
            "Command Error: {}",
            error_message
        )));
    }
}

pub fn check_channel(
    rcvr: &Receiver<Result<(), CliError>>,
    rdy: &mut bool,
) -> Result<(), CliError> {
    if !*rdy {
        match rcvr.try_recv() {
            Ok(Ok(())) => *rdy = true,
            Ok(Err(err)) => return Err(err),
            Err(TryRecvError::Disconnected) => {
                return Err(CliError::Message("channel disconnected".to_string()))
            }
            _ => {}
        }
    }

    Ok(())
}