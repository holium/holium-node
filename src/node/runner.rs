use std::{io, process::Command};

use crate::cli::{printer::print_to_cli, tmux::TmuxManager};

pub struct NodeRunner;

impl NodeRunner {
    pub fn start(&self, server_id: &str, node_port: u16, urbit_port: u16) -> io::Result<()> {
        let session_name = format!("{}-api", server_id);
        let is_instance_running = TmuxManager::is_session_running(server_id);
        let is_node_running = TmuxManager::is_session_running(session_name.as_str());
        if !is_node_running & is_instance_running {
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

            TmuxManager::create_session(session_name.as_str(), None)?;
            TmuxManager::send_command(session_name.as_str(), &command)?;
        } else {
            if !is_instance_running {
                print_to_cli(format!(
                    "instance '{}'     not running, skipping node start",
                    server_id
                ));
            } else {
                print_to_cli(format!("node     '{}' already running", session_name));
            }
        }

        Ok(())
    }
    pub fn stop(&self, server_id: &str) -> io::Result<()> {
        TmuxManager::terminate_session(format!("{}-api", server_id).as_str())?;
        Ok(())
    }
}
