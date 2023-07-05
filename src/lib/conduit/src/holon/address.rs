use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Address {
    pub proto_version: i8,
    pub ip: String,
    pub port: i16,
    pub last_ping: i32,
}

impl Address {
    pub fn new(proto_version: i8, ip: String, port: i16) -> Self {
        Address {
            proto_version,
            ip,
            port,
            last_ping: 0,
        }
    }

    pub fn get_addr(&self) -> String {
        return format!("{}:{}", self.ip, self.port);
    }
}
