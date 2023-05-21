use std::io;

pub mod urbit;

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
