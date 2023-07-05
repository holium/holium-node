use aes_gcm::{aead::generic_array::GenericArray, Aes256Gcm, KeyInit};
use rand::{self, Rng};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use x25519_dalek::{PublicKey, SharedSecret, StaticSecret};

use aes::Aes256;

pub struct PeerKeyPair {
    pub peer_xpub: [u8; 32],
    pub our_xpub: [u8; 32],
    shared_secret: Option<SharedSecret>,
}
impl Clone for PeerKeyPair {
    fn clone(&self) -> Self {
        PeerKeyPair {
            our_xpub: self.our_xpub.clone(),
            peer_xpub: self.peer_xpub.clone(),
            shared_secret: None,
        }
    }
}
#[allow(unconditional_recursion)]
impl Serialize for PeerKeyPair {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        PeerKeyPair {
            our_xpub: self.our_xpub.clone(),
            peer_xpub: self.peer_xpub.clone(),
            shared_secret: None,
        }
        .serialize(serializer)
    }
}
#[allow(unconditional_recursion)]
impl<'de> Deserialize<'de> for PeerKeyPair {
    fn deserialize<D>(deserializer: D) -> Result<PeerKeyPair, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let peer_keypair = PeerKeyPair::deserialize(deserializer)?;
        Ok(PeerKeyPair {
            our_xpub: peer_keypair.our_xpub,
            peer_xpub: peer_keypair.peer_xpub,
            shared_secret: None,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Address {
    pub proto_version: i8,
    pub ip: String,
    pub port: String,
    pub last_ping: i32,
}

impl Address {
    pub fn new(proto_version: i8, ip: String, port: String) -> Self {
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

#[derive(Clone, Serialize, Deserialize)]
pub struct Peer {
    pub generation: i8, // the generation of the peer, should be incremented when the keypair is updated
    pub hid: String,    // the holon id of the peer
    pub keypair: PeerKeyPair,
    pub address: Address, // the ip address and port of the peer
}

impl Peer {
    pub fn new(generation: i8, hid: String, peer_xpub: [u8; 32], address: Address) -> Self {
        let secret = StaticSecret::random_from_rng(OsRng);
        let our_xpub = PublicKey::from(&secret);
        let their_xpub = PublicKey::from(peer_xpub);
        let shared_secret = secret.diffie_hellman(&their_xpub);
        return Peer {
            generation,
            hid,
            keypair: PeerKeyPair {
                our_xpub: our_xpub.as_bytes().to_owned(),
                peer_xpub: peer_xpub,
                shared_secret: Some(shared_secret),
            },
            address,
        };
    }

    pub fn get_addr(&self) -> String {
        self.address.get_addr()
    }

    // encrypt a message with the shared secret
    pub fn encrypt(&self, message: &[u8]) -> Vec<u8> {
        let shared_secret = &self.keypair.shared_secret.unwrap().to_bytes();
        let mut rng = rand::thread_rng();
        let iv: [u8; 16] = rng.gen();
        let cipher = Aes256::new_from_slices(&shared_secret, &iv).unwrap();

        let mut buffer = [0u8; 32]; // Buffer size must be multiple of block size (16 for AES)
        buffer[..message.len()].copy_from_slice(message);
        let cipher_text = cipher.encrypt(&mut buffer, message.len()).unwrap();
        return cipher_text;
    }

    // decrypt with shared_secret and nonce
    pub fn decrypt(&self, message: &[u8]) -> Vec<u8> {
        let shared_secret = &self.keypair.shared_secret.unwrap().to_bytes();
        let mut rng = rand::thread_rng();
        let iv: [u8; 16] = rng.gen();

        let cipher = Aes256Cbc::new_from_slices(&shared_secret, &iv).unwrap();
        let decrypted_data = cipher.decrypt_vec(message).unwrap();
        return decrypted_data;
    }
}
