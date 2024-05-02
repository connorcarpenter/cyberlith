use std::time::Duration;

use logging::{info, warn};
use openssh::Session;

use crate::{
    utils::{run_ssh_command, run_ssh_raw_command},
    CliError,
};

pub async fn instance_init(session: &Session) -> Result<(), CliError> {
    setup_docker(&session).await?;

    Ok(())
}

async fn setup_docker(session: &Session) -> Result<(), CliError> {
    //info!("# update");
    run_ssh_command(&session, "sudo apt update").await?;

    //info!("# install dependencies");
    run_ssh_command(&session, "sudo apt install ca-certificates curl -y").await?;

    //info!("# install keyring");
    run_ssh_command(&session, "sudo install -m 0755 -d /etc/apt/keyrings").await?;

    //info!("# download GPG key and install");
    run_ssh_command(&session, "sudo curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc").await?;

    //info!("# set permissions on keyring");
    run_ssh_command(&session, "sudo chmod a+r /etc/apt/keyrings/docker.asc").await?;

    //info!("# add docker to apt sources?");
    run_ssh_raw_command(&session, "echo \"deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/ubuntu $(. /etc/os-release && echo \"$VERSION_CODENAME\") stable\" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null").await?;

    //info!("# update");
    run_ssh_command(&session, "sudo apt update").await?;

    //info!("# install docker packages");
    run_ssh_command(&session, "sudo apt install docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin -y").await?;

    //info!("# add user to docker group");
    loop {
        match run_ssh_command(&session, "sudo usermod -aG docker root").await {
            Ok(()) => {
                break;
            }
            Err(err) => {
                warn!("error adding user to docker group: {:?}", err);
                info!("retrying after 5 seconds..");
                executor::smol::Timer::after(Duration::from_secs(5)).await;
            }
        }
    }

    info!("# test that docker works without sudo");
    run_ssh_command(&session, "docker version").await?;

    Ok(())
}

// // this is no loger necessary because Vultrs Firewall Group config handles it, but it may be useful to reference in the future
// async fn setup_iptables(session: &Session) -> Result<(), CliError> {
//
//     info!("Setting up IPTables");
//
//     info!("# allow established connections");
//     run_ssh_command(&session, "sudo iptables -A INPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT").await?;
//
//     info!("# allow ssh");
//     run_ssh_command(&session, "sudo iptables -A INPUT -p tcp --dport ssh -j ACCEPT").await?;
//
//     info!("# allow loopback");
//     run_ssh_command(&session, "sudo iptables -I INPUT 1 -i lo -j ACCEPT").await?;
//
//     info!("# allow port 80 (content server)");
//     run_ssh_command(&session, "sudo iptables -A INPUT -p tcp --dport 80 -j ACCEPT").await?;
//
//     info!("# allow port 14197 (gateway)");
//     run_ssh_command(&session, "sudo iptables -A INPUT -p tcp --dport 14197 -j ACCEPT").await?;
//
//     info!("# allow port 14200 (session signal)");
//     run_ssh_command(&session, "sudo iptables -A INPUT -p tcp --dport 14200 -j ACCEPT").await?;
//
//     info!("# allow port 14201 (session webrtc)");
//     run_ssh_command(&session, "sudo iptables -A INPUT -p udp --dport 14201 -j ACCEPT").await?;
//
//     info!("# allow port 14203 (world signal)");
//     run_ssh_command(&session, "sudo iptables -A INPUT -p tcp --dport 14203 -j ACCEPT").await?;
//
//     info!("# allow port 14204 (world webrtc)");
//     run_ssh_command(&session, "sudo iptables -A INPUT -p udp --dport 14204 -j ACCEPT").await?;
//
//     Ok(())
// }
