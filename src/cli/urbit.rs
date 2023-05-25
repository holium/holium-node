use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

use crate::cli::printer::print_to_cli;
use crate::cli::tmux::TmuxManager;

/// This is an abstract trait for the CLI interface. Instance
/// types should implement this trait.
pub trait Instance {
    type UpdateOptions;

    fn download_and_setup(&self, binary_name: &str) -> io::Result<()>;
    fn boot(&self, server_id: &str, fake: bool, key: Option<String>, port: u16) -> io::Result<()>;
    fn start(&self, server_id: &str, port: u16) -> io::Result<()>;
    fn stop(&self, server_id: &str, port: u16) -> io::Result<()>;
    fn clean(&self, server_id: &str, method: &str) -> io::Result<()>;
    fn info(&self, server_id: &str) -> io::Result<()>;
    fn logs(&self, server_id: &str, attach: bool, num_of_lines: i32) -> io::Result<()>;
    fn upgrade(&self, server_id: &str, options: Self::UpdateOptions) -> io::Result<()>;
    fn apps(&self, server_id: &str) -> io::Result<()>;
    fn app(&self, server_id: &str, app_name: &str) -> io::Result<()>;
    fn version(&self) -> io::Result<()>;
}

const BINARY_URL: &str = if cfg!(target_os = "macos") {
    "https://urbit.org/install/macos-x86_64/latest"
} else if cfg!(target_os = "linux") {
    "https://urbit.org/install/linux-x86_64/latest"
} else {
    panic!("Unsupported platform");
};

pub fn symlink_urbit_binary(server_id: String) -> io::Result<String> {
    let symlinked_urbit = format!("{}_urbit", server_id);
    TmuxManager::send_command(
        &server_id,
        &Command::new("ln")
            .arg("-s")
            .arg("urbit")
            .arg(&symlinked_urbit),
    )?;
    Command::new("ln")
        .arg("-s")
        .arg("urbit")
        .arg(&symlinked_urbit)
        .output()?;
    Ok(symlinked_urbit)
}

pub struct UrbitInstance;

pub struct UrbitUpdateOptions {
    pub update_vere: Option<bool>,
    pub update_urbit: Option<bool>,
    pub update_all: Option<bool>,
}

impl UrbitInstance {
    pub fn has_urbit_binary(&self) -> bool {
        Path::new("./urbit").exists()
    }

    pub fn args_to_file(&self, server_id: &str, args: &str) -> io::Result<()> {
        fs::write(format!("ships/.{}.params", server_id), args)?;
        Ok(())
    }

    pub fn fake_to_file(&self, server_id: &str) -> io::Result<()> {
        fs::write(format!("ships/.{}.fake", server_id), true.to_string())?;
        Ok(())
    }

    pub fn clear_params_file(&self, server_id: &str) -> io::Result<()> {
        fs::write(format!("ships/.{}.params", server_id), "")?;
        Ok(())
    }

    pub fn get_current_args(&self, server_id: &str) -> io::Result<Vec<String>> {
        let args = fs::read_to_string(format!("ships/.{}.params", server_id))?;
        let args = args.split("\n").map(|s| s.to_string()).collect();
        Ok(args)
    }
}

impl Instance for UrbitInstance {
    type UpdateOptions = UrbitUpdateOptions;

    fn download_and_setup(&self, binary_name: &str) -> io::Result<()> {
        if !Path::new(binary_name).exists() {
            println!("Downloading Urbit binary...");
            // Download the latest Urbit binary
            Command::new("curl")
                .arg("-L")
                .arg(BINARY_URL)
                .arg("-o")
                .arg("urbit.tar.gz")
                .status()?;

            // Extract the file
            Command::new("tar")
                .arg("zxvf")
                .arg("urbit.tar.gz")
                .arg("-s")
                .arg("/.*/urbit/")
                .status()?;

            // Make the binary executable
            Command::new("chmod")
                .arg("+x")
                .arg(binary_name)
                .output()
                .expect("Failed to execute command");

            // Make ships folder
            Command::new("mkdir")
                .arg("ships")
                .output()
                .expect("Failed to execute command");

            // remove the tar file
            fs::remove_file("urbit.tar.gz")?;
        }
        Ok(())
    }

    fn boot(&self, server_id: &str, fake: bool, key: Option<String>, port: u16) -> io::Result<()> {
        if !Path::new(format!("ships").as_str()).exists() {
            Command::new("mkdir")
                .arg("ships")
                .output()
                .expect("Failed to execute command");
        }
        if !Path::new(format!("ships/{}", server_id).as_str()).exists() {
            // create screen session
            TmuxManager::create_session(&server_id, None)?;
            let symlinked_urbit = symlink_urbit_binary(server_id.to_string())?;
            // smylink server_id_urbit to urbit
            TmuxManager::send_command(
                &server_id,
                &Command::new("ln")
                    .arg("-s")
                    .arg("urbit")
                    .arg(&symlinked_urbit),
            )?;

            let mut command = Command::new(format!("./{}", symlinked_urbit));
            // execute urbit in screen session
            if fake {
                command.arg("-F");
                command.arg(&server_id.to_string());
                command.arg("-c").arg(format!("ships/{}", server_id));
            } else if let Some(key) = &key {
                command.arg("-w").arg(&server_id);
                command.arg("-G").arg(key);
                command.arg("-c").arg(format!("ships/{}", server_id));
            }
            command.arg("--http-port").arg(&port.to_string());
            TmuxManager::send_command(&server_id, &command)?;
            // save args to file
            let mut args = Vec::new();
            args.push(format!("server_id: {}", server_id.to_string()));
            args.push(format!("urbit_port: {}", port.to_string()));
            args.push(format!("all: {:?}", command.get_args().collect::<Vec<_>>()));
            if fake {
                self.fake_to_file(server_id)?;
            }
            let args = args.join("\n");
            self.args_to_file(server_id, &args)?;
        } else {
            print_to_cli(format!("instance '{}'     already booted", server_id));
        }
        Ok(())
    }

    fn start(&self, server_id: &str, port: u16) -> io::Result<()> {
        if !self.has_urbit_binary() {
            print_to_cli("No urbit binary found. Please run `hol install` to install the binary.");
        }
        // Check if a session is running
        let is_running = TmuxManager::is_session_running(server_id);
        if !is_running {
            TmuxManager::create_session(&server_id, None)?;
            let symlinked_urbit = symlink_urbit_binary(server_id.to_string())?;
            let mut command = Command::new(format!("./{}", symlinked_urbit));

            // check if a folder with the server ID exists
            if !Path::new(format!("ships/{}", server_id).as_str()).exists() {
                print_to_cli(format!("Identity {} is not booted", server_id));
            } else {
                command.arg(format!("ships/{}", server_id));
            }

            command.arg("--http-port").arg(&port.to_string());
            TmuxManager::send_command(&server_id, &command)?;
            print_to_cli(format!(
                "Started urbit instance with args: {:?}",
                command.get_args().collect::<Vec<_>>()
            ));
            // save args to file
            let mut args = Vec::new();
            args.push(format!("server_id: {}", server_id.to_string()));
            args.push(format!("urbit_port: {}", port.to_string()));
            args.push(format!("all: {:?}", command.get_args().collect::<Vec<_>>()));
            let args = args.join("\n");
            self.args_to_file(server_id, &args)?;
        } else {
            print_to_cli(format!("instance '{}'     already running", server_id));
        }
        Ok(())
    }

    fn stop(&self, server_id: &str, port: u16) -> io::Result<()> {
        TmuxManager::terminate_session(&server_id)?;
        self.clear_params_file(server_id)?;
        print_to_cli(format!(
            "Stopped Urbit instance with server ID {} on port {}",
            server_id, port
        ));
        Ok(())
    }

    fn clean(&self, server_id: &str, method: &str) -> std::io::Result<()> {
        println!("{} {}", server_id, method);
        println!("yeo yeo yeo");
        let is_running = TmuxManager::is_session_running(server_id);
        if !is_running {
            println!("not running");
        } else {
            println!("running");
// Running this command via Tmux is what I DON'T want to do...
            TmuxManager::send_dojo_command(server_id, "|exit")?;
// and what I really want to do is something more like this via lens:
            /*
            let exit_status = helper::ship_exit(server_id)
            .await
            .expect("Could not get exit status");
            print_to_cli(exit_status);
            */
            let symlinked_urbit_binary = format!("./{}_urbit", server_id.to_string());
            let mut command = Command::new(symlinked_urbit_binary);
            command.arg("pack");
            command.arg(format!("ships/{}", server_id));
            TmuxManager::send_command(&server_id, &command)?;
        }
        Ok(())
    }

    fn info(&self, server_id: &str) -> std::io::Result<()> {
        print_to_cli(TmuxManager::list_sessions()?);
        print_to_cli(self.get_current_args(server_id).unwrap().join("\n"));
        Ok(())
    }

    fn logs(&self, server_id: &str, attach: bool, num_of_lines: i32) -> std::io::Result<()> {
        println!("{}, {}, {}", server_id, attach, num_of_lines);
        todo!()
    }

    fn upgrade(&self, server_id: &str, options: Self::UpdateOptions) -> std::io::Result<()> {
        println!("{}, update_all={:?}", server_id, options.update_all);
        Ok(())
    }

    fn apps(&self, server_id: &str) -> std::io::Result<()> {
        println!("{}", server_id);
        TmuxManager::send_dojo_command(server_id, "+vats")?;
        Ok(())
    }

    fn app(&self, server_id: &str, app_name: &str) -> std::io::Result<()> {
        println!("{}, {}", server_id, app_name);
        todo!()
    }

    fn version(&self) -> std::io::Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urbit_instance() {
        let urbit = UrbitInstance;
        let options = UrbitUpdateOptions {
            update_all: Some(false),
            update_urbit: Some(false),
            update_vere: Some(false),
        };

        // Test download_and_setup_binary
        assert!(urbit.download_and_setup("urbit_binary").is_ok());

        // Test boot
        assert!(urbit
            .boot("server_id", false, Some("key".to_string()), 12345)
            .is_ok());

        // Test start
        assert!(urbit.start("server_id", 12345).is_ok());

        // Test stop
        assert!(urbit.stop("server_id", 12345).is_ok());

        // Test upgrade
        assert!(urbit.upgrade("server_id", options).is_ok());
    }
}
