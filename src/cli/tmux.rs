use std::process::Command;
use std::io;

pub struct TmuxManager {}

impl TmuxManager {
    // Create a new detached tmux session
    pub fn create_session(session_name: &str, init_command: Option<&Command>) -> io::Result<()> {
        let mut tmux_session = Command::new("tmux")
            .arg("new-session")
            .arg("-d")
            .arg("-s")
            .arg(session_name)
            .arg("-c")
            .arg(".")
            .spawn()?;
        let _ = tmux_session.wait()?;
        if let Some(command) = init_command {
            let _ = TmuxManager::send_command(session_name, command);
        }
        Ok(())
    }

    // Send a command to a detached tmux session
    pub fn send_command(session_name: &str, command: &Command) -> io::Result<()> {
        let command_str = format!("{:?}", command);
        let mut command_session = Command::new("tmux")
            .arg("send-keys")
            .arg("-t")
            .arg(session_name)
            .arg(command_str)
            .arg("Enter")  
            .spawn()?;
        let _ = command_session.wait()?;
        Ok(())
    }

    // List current tmux sessions
    pub fn list_sessions() -> io::Result<String> {
        let output = Command::new("tmux")
            .arg("list-sessions")
            .output()?;
        Ok(String::from_utf8(output.stdout).unwrap())
    }

    pub fn is_session_running(session_name: &str) -> bool {
        let output = TmuxManager::list_sessions().unwrap();
        output.contains(session_name)
    }

    // Terminate a tmux session
    pub fn terminate_session(session_name: &str) -> io::Result<()> {
        let mut command_session = Command::new("tmux")
            .arg("kill-session")
            .arg("-t")
            .arg(session_name)
            .spawn()?;
        let _ = command_session.wait()?;
        Ok(())
    }
}