use std::io;
use std::process::Command;

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
        // convert the command to a single string
        let command_str = format!("{:?}", command).replace("\"", "");
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

    pub fn send_dojo_command(session_name: &str, input_str: &str) -> io::Result<()> {
        let mut command_session = Command::new("tmux")
            .arg("send-keys")
            .arg("-t")
            .arg(session_name)
            .arg(input_str)
            .arg("Enter")
            .spawn()?;
        let _ = command_session.wait()?;
        Ok(())
    }

    // List current tmux sessions
    pub fn list_sessions() -> io::Result<String> {
        let output = Command::new("tmux").arg("list-sessions").output()?;
        Ok(String::from_utf8(output.stdout).unwrap())
    }

    pub fn is_session_running(session_name: &str) -> bool {
        let output = TmuxManager::list_sessions().unwrap();
        output.contains(session_name)
    }

    // pub fn attach_to_logs(session_name: &str) -> io::Result<()> {
    //     let mut command_session = Command::new("tmux")
    //         .arg("attach-session")
    //         .arg("-t")
    //         .arg(session_name)
    //         .spawn()?;
    //     let _ = command_session.wait()?;
    //     Ok(())
    // }

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
