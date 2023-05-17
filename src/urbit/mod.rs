use std::path::Path;
use std::fs;
use std::process::{Command};

use crate::cli::printer::print_to_cli;
use crate::cli::tmux::TmuxManager;


const BINARY_URL: &str = if cfg!(target_os = "macos") {
    "https://urbit.org/install/macos-x86_64/latest"
} else if cfg!(target_os = "linux") {
    "https://urbit.org/install/linux-x86_64/latest"
} else {
    panic!("Unsupported platform");
};

pub fn has_urbit_binary() -> bool {
    Path::new("./urbit").exists()
}

pub fn download_and_setup_binary(binary_name: &str) -> std::io::Result<()> {
    if !Path::new(binary_name).exists() {
        println!("Downloading Urbit binary...");
        // Download the latest Urbit binary
        Command::new("curl")
            .arg("-L")
            .arg(BINARY_URL)
            .arg("-o")
            .arg("urbit.tar.gz").status()?;

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
            
        Command::new("mkdir").arg("ships").output().expect("Failed to execute command");

    // remove the tar file
    fs::remove_file("urbit.tar.gz")?;
  }
  Ok(())
}

pub fn boot_urbit(server_id: &str, fake: bool, key: Option<String>, urbit_port: u16) -> std::io::Result<()> {
    if !Path::new(format!("ships/{}", server_id).as_str()).exists() {
        
        // create screen session
        TmuxManager::create_session(&server_id, None)?;

        let mut command = Command::new("screen");
        // execute urbit in screen session
        command.arg("-X").arg(&server_id.to_string()).arg("./urbit");
        if fake {
            command.arg("-F");
            command.arg(&server_id.to_string());
            command.arg("-c").arg(format!("ships/{}", server_id));
        } else if let Some(key) = &key {
            command.arg("-w").arg(&server_id);
            command.arg("-G").arg(key);
        }
        command.arg("--exit"); // exit after booting
        command.arg("--http-port").arg(&urbit_port.to_string());
        TmuxManager::send_command(&server_id, &command)?;
        let child = command.spawn().expect("Failed to execute command");
        print_to_cli(format!("Started Urbit instance with PID {}", child.id()));
        // set lock file for identity
        fs::write(format!("ships/{}.lock", server_id), "").expect("Unable to write file");
    } else {
       print_to_cli(format!("Identity {} is already booted", server_id));
    }
    Ok(())
}

pub fn start_urbit(server_id: &str, urbit_port: u16) -> std::io::Result<()> {
    // Check if a session is running
    let is_running = TmuxManager::is_session_running(server_id);
    if !is_running {
        TmuxManager::create_session(&server_id, None)?;
        let mut command = Command::new("./urbit");

        // check if a folder with the server ID exists
        if !Path::new(format!("ships/{}", server_id).as_str()).exists() {
            print_to_cli(format!("Identity {} is not booted", server_id));
        } else {
            command.arg(format!("ships/{}", server_id));
        }

        command.arg("--http-port").arg(&urbit_port.to_string());
        print_to_cli(format!("Starting urbit with args: {:?}", command.get_args().collect::<Vec<_>>()));
        TmuxManager::send_command(&server_id, &command)?;
        Ok(())
    } else {
        print_to_cli(format!("Identity {} is already running", server_id));
        Ok(())
    }
}

pub fn stop_urbit (server_id: &str, urbit_port: u16) -> std::io::Result<()> {
    TmuxManager::terminate_session(&server_id)?;
    //  let output = Command::new("bash")
    //     .arg("-c")
    //     .arg(format!("ps -eo pid,comm,args | grep 'urbit' | grep -v grep | grep '{}'", server_id))
    //     .output()
    //     .expect("Failed to execute command");

    // let output = String::from_utf8(output.stdout).expect("Not UTF-8");
    // println!("output:\n{}", output);
    // for line in output.lines() {
    //     let mut parts = line.trim().splitn(3, ' ');
    //     let pid = parts.next().unwrap().parse::<i32>().unwrap();
    //     // kill the process
    //     let _ = std::process::Command::new("kill")
    //         .arg("-9")
    //         .arg(pid.to_string())
    //         .output();
    // }

    print_to_cli(format!("Stopped Urbit instance with server ID {} on port {}", server_id, urbit_port));
    Ok(())
}