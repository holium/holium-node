## Peers module


### Tables

```rs
pub enum Transport {
  UDP = 'udp'
  TCP = 'tcp'
}

pub struct Address {
    pub version: i8,
    pub ip: String,
    pub port: String,
    pub transport: Transport
    pub last_ping: i32
}

pub struct Peer {
    pub version: i8,
    pub hid: String,
    pub addr: String,
    pub xpub: String,
    pub addresses: [Address]
}

```