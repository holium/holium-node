use std::path::Path;
use std::fs;
use std::process::Command;

const BINARY_URL: &str = if cfg!(target_os = "macos") {
    "https://urbit.org/install/macos-x86_64/latest"
} else if cfg!(target_os = "linux") {
    "https://urbit.org/install/linux-x86_64/latest"
} else {
    panic!("Unsupported platform");
};

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

pub fn start_urbit(server_id: &str, fake: bool, key: Option<String>, urbit_port: u16) -> std::io::Result<Command> {
    let mut command = Command::new("./urbit");
    // check if a folder with the server ID exists
    command.arg("-d");
    if !Path::new(format!("ships/{}", server_id).as_str()).exists() {
        if fake {
            command.arg("-F");
            command.arg(&server_id.to_string());
            command.arg("-c").arg(format!("ships/{}", server_id));
        } else if let Some(key) = &key {
            command.arg("-w").arg(&server_id);
            command.arg("-G").arg(key);
        }
    } 
    
    
    if Path::new(format!("ships/{}", server_id).as_str()).exists() {
        command.arg("-q");
        command.arg(format!("ships/{}", server_id));
    }

    command.arg("--http-port").arg(&urbit_port.to_string());
    println!("Starting urbit with args: {:?}", command.get_args().collect::<Vec<_>>());
    Ok(command)
}