use std::{io, process::Command};

use crate::cli::tmux::TmuxManager;

pub struct NodeRunner;

impl NodeRunner {
    pub fn start(&self, server_id: &str, node_port: u16, urbit_port: u16) -> io::Result<()> {
        let mut command = Command::new("cargo");

        command
            .arg("run")
            .arg("--bin")
            .arg("api")
            .arg("--")
            .arg("--urbit-port")
            .arg(&urbit_port.to_string())
            .arg("--node-port")
            .arg(&node_port.to_string());

        let session_name = format!("{}-api", server_id);
        TmuxManager::create_session(session_name.as_str(), None)?;
        TmuxManager::send_command(session_name.as_str(), &command)?;
        Ok(())
    }
    pub fn stop(&self, server_id: &str) -> io::Result<()> {
        TmuxManager::terminate_session(format!("{}-api", server_id).as_str())?;
        Ok(())
    }
}
