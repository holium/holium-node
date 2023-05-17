use super::urbit;

pub fn start_instance(server_id: &str, urbit_port: u16) -> std::io::Result<()> {
  if !urbit::has_urbit_binary() {
    // throw error
    println!("No urbit binary found. Please run `hol install` to install the binary.");
  }
  let mut command = urbit::start_urbit(&server_id, true, None, urbit_port).unwrap();
  command.spawn().expect("Failed to start Urbit instance");
  Ok(())
}

pub fn stop_instance(server_id: &str, urbit_port: u16) -> std::io::Result<()> {
  println!("Stopping instance {} on port {}", server_id, urbit_port);
  urbit::stop_urbit(server_id, urbit_port)?;
  Ok(())
}