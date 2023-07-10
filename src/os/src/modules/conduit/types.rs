use serde::{Deserialize, Serialize};

use crate::holon::Address;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConduitPeer {
    pub hid: String,
    pub pubkey: [u8; 32],
    pub addr: Address,
    // xpriv: Option<String>,
}

impl ConduitPeer {
    pub fn new(hid: String, addr: Address, pubkey: [u8; 32]) -> Self {
        ConduitPeer {
            hid,
            addr,
            pubkey,
            // xpriv,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConduitPacket {
    pub from: ConduitPeer, // our
    pub to: ConduitPeer,   // peer
    pub seq_num: u64,
    pub data: Vec<u8>,
    pub signature: String,
}

impl ConduitPacket {
    pub fn new(
        from: &ConduitPeer,
        to: &ConduitPeer,
        seq_num: u64,
        data: Vec<u8>,
        signature: String,
    ) -> Self {
        ConduitPacket {
            from: from.clone(),
            to: to.clone(),
            seq_num,
            data,
            signature,
        }
    }

    pub fn serialize(&self) -> Result<Vec<u8>, Box<bincode::ErrorKind>> {
        bincode::serialize(self)
    }

    pub fn deserialize(data: &[u8]) -> Result<ConduitPacket, Box<bincode::ErrorKind>> {
        bincode::deserialize(data)
    }
}
