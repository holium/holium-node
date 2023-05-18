pub struct API {}

impl API {
    // pub fn start(&self, server_id: &str) -> API {
    //     TmuxManager::create_session(&(server_id.to_string() + "-api"), None)?;
    //     let mut command = Command::new("cargo");
    //     command.arg("run").arg("--bin").arg("my_api_server");
    //     TmuxManager::send_command(&(server_id.to_string() + "-api"), &command)?;
    //     let child = command.spawn().expect("Failed to execute command");
    //     print_to_cli(format!("Started API server with PID {}", child.id()));
    // }
}
