use std::collections::HashMap;
use std::error::Error;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

async fn get_pid(server_id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let pid = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "ps aux | grep \"{}_urbit ships\" | grep -v grep | awk '{{print $2}}' | tr -d '\n'",
            server_id
        ))
        .output()
        .expect("failed to find pid");

    let pid_str = String::from_utf8(pid.stdout)?;
    if pid_str.is_empty() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to get pid",
        )));
    }
    Ok(pid_str)
}

pub async fn get_pid_and_loopback(
    server_id: String,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let pid_str = get_pid(&server_id).await?;
    let cmd_str = if cfg!(target_os = "macos") {
        format!(
          "lsof -n -P -p {} |  grep '127.0.0.1' | grep urbit | grep -i listen | cut -d':' -f2 | cut -c -5 | tr -d '\n'",
          pid_str
        )
    } else if cfg!(target_os = "linux") {
        format!(
          "lsof -n -P -p {} |  grep '127.0.0.1' | grep -i listen | cut -d':' -f2 | cut -c -5 | tr -d '\n'",
          pid_str
        )
    } else {
        panic!("Unsupported platform");
    };
    let loopback = Command::new("sh")
        .arg("-c")
        .arg(cmd_str)
        .output()
        .expect("failed to find loopback");

    let loopback_str = String::from_utf8_lossy(&loopback.stdout).to_string();

    if loopback_str.is_empty() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to get loopback",
        )));
    }

    Ok((pid_str, loopback_str))
}

pub fn graceful_exit(
    server_id: &str,
    max_wait_seconds: u64,
) -> Result<HashMap<&str, bool>, Box<dyn Error>> {
    let pid_str = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(get_pid(server_id))?;

    // Attempt a graceful exit with SIGTERM (kill -15)
    let _ = Command::new("kill").arg("-15").arg(&pid_str).output()?;

    // Wait for the process to exit gracefully
    for _ in 0..max_wait_seconds {
        sleep(Duration::from_millis(500));
        // Check if the process is still running
        match tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(get_pid(server_id))
        {
            Ok(_) => continue,
            Err(_) => return Ok([("graceful_exit", true)].iter().cloned().collect()), // Process has exited
        }
    }

    // If the process is still running after max_wait_seconds, force kill it with SIGKILL (kill -9)
    let _ = Command::new("kill").arg("-9").arg(&pid_str).output()?;

    Ok([("graceful_exit", false)].iter().cloned().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    // use tokio::runtime::Runtime;

    fn setup() {
        println!("Boot up a fake zod...");
    }

    #[test]
    fn test_urbit_graceful_exit() {
        setup();

        let server_id = "zod";
        let max_wait_seconds = 5;

        let result = graceful_exit(server_id, max_wait_seconds).unwrap();
        if result == [("graceful_exit", true)].iter().cloned().collect() {
            println!("Urbit process exited gracefully");
        } else {
            println!("Urbit process did not exit gracefully");
        }
    }
}
