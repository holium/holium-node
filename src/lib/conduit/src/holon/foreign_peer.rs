// crate imports
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use x25519_dalek::{PublicKey, SharedSecret, StaticSecret};

// project imports
use super::{
    address::Address,
    state::{PrintStateChangeListener, StateChangeListener},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ForeignPeerState {
    Discovered,
    HandshakeSent,
    HandshakeReceived,
    Verified,
    Rejected,
}

pub struct ForeignPeerEntry {
    pub epoch: i8, // the epoch of the peer, should be incremented when the keypair is updated
    pub hid: String, // the holon id of the peer
    pub address: Address, // the ip address and port of the peer
    pub pubkey: [u8; 32], // the public key of the peer
}

// #[derive(Clone, Serialize, Deserialize)]
pub struct ForeignPeer {
    pub epoch: i8, // the epoch of the peer, should be incremented when the keypair is updated
    pub hid: String, // the holon id of the peer
    pub address: Address, // the ip address and port of the peer
    pub pubkey: [u8; 32], // the public key of the peer
    pub state: ForeignPeerState,
    pub state_listener: Rc<dyn StateChangeListener>,
    shared_secret: Option<SharedSecret>,
}

impl ForeignPeer {
    pub fn new(epoch: i8, hid: String, address: Address, peer_pubkey: [u8; 32]) -> Self {
        let listener = Rc::new(PrintStateChangeListener);

        return ForeignPeer {
            epoch,
            hid,
            pubkey: peer_pubkey,
            address,
            shared_secret: None,
            state: ForeignPeerState::Discovered,
            state_listener: listener,
        };
    }

    pub fn get_addr(&self) -> String {
        self.address.get_addr()
    }

    pub fn send_handshake(&mut self, private_key: &[u8; 32]) -> Result<(), &'static str> {
        match self.state {
            ForeignPeerState::Discovered => {
                // let signature = sign_message(private_key, &self.pubkey);
                // Send handshake message to our_peer
                // self.send((self.pubkey.to_vec(), signature));
                self.state = ForeignPeerState::HandshakeSent;
                Ok(())
            }
            _ => Err("Invalid state transition"),
        }
    }

    pub fn init_shared_secret(&mut self, secret: &StaticSecret) -> Result<(), &'static str> {
        match self.state {
            ForeignPeerState::Verified => {
                let their_pubkey = PublicKey::from(self.pubkey);
                let shared_secret = secret.diffie_hellman(&their_pubkey);
                self.shared_secret = Some(secret.diffie_hellman(&their_pubkey));
                self.state = ForeignPeerState::HandshakeSent;
                Ok(())
            }
            _ => Err("Invalid state transition"),
        }
    }

    // encrypt a message with the shared secret
    // pub fn encrypt(&self, message: &[u8]) -> Vec<u8> {
    //     let shared_secret = &self.keypair.shared_secret.unwrap().to_bytes();
    //     let mut rng = rand::thread_rng();
    //     let iv: [u8; 16] = rng.gen();
    //     let cipher = Aes256::new_from_slices(&shared_secret, &iv).unwrap();

    //     let mut buffer = [0u8; 32]; // Buffer size must be multiple of block size (16 for AES)
    //     buffer[..message.len()].copy_from_slice(message);
    //     let cipher_text = cipher.encrypt(&mut buffer, message.len()).unwrap();
    //     return cipher_text;
    // }

    // // decrypt with shared_secret and nonce
    // pub fn decrypt(&self, message: &[u8]) -> Vec<u8> {
    //     let shared_secret = &self.keypair.shared_secret.unwrap().to_bytes();
    //     let mut rng = rand::thread_rng();
    //     let iv: [u8; 16] = rng.gen();

    //     let cipher = Aes256Cbc::new_from_slices(&shared_secret, &iv).unwrap();
    //     let decrypted_data = cipher.decrypt_vec(message).unwrap();
    //     return decrypted_data;
    // }
}

// Implement clone without shared_secret
impl Clone for ForeignPeer {
    fn clone(&self) -> Self {
        ForeignPeer {
            epoch: self.epoch,
            hid: self.hid.clone(),
            address: self.address.clone(),
            pubkey: self.pubkey.clone(),
            shared_secret: None,
            state: self.state.clone(),
            state_listener: self.state_listener.clone(),
        }
    }
}

// implement serialize without shared_secret
#[allow(unconditional_recursion)]
impl Serialize for ForeignPeer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        ForeignPeer {
            epoch: self.epoch,
            hid: self.hid.clone(),
            address: self.address.clone(),
            pubkey: self.pubkey.clone(),
            shared_secret: None,
            state: self.state.clone(),
            state_listener: self.state_listener.clone(),
        }
        .serialize(serializer)
    }
}

// implement serialize without shared_secret
#[allow(unconditional_recursion)]
impl<'de> Deserialize<'de> for ForeignPeer {
    fn deserialize<D>(deserializer: D) -> Result<ForeignPeer, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let peer = ForeignPeer::deserialize(deserializer)?;
        Ok(ForeignPeer {
            epoch: peer.epoch,
            hid: peer.hid.clone(),
            address: peer.address.clone(),
            pubkey: peer.pubkey.clone(),
            shared_secret: None,
            state: peer.state,
            state_listener: peer.state_listener,
        })
    }
}
