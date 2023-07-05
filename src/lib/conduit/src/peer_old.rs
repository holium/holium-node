use aes_gcm::{aead::generic_array::GenericArray, Aes256Gcm, KeyInit};
use rand::{self, Rng};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use x25519_dalek::{PublicKey, SharedSecret, StaticSecret};

use aes::Aes256;

// type Aes256Cbc = Cbc<Aes256, Pkcs7>;
pub struct PeerKeyPair {
    pub peer_xpub: [u8; 32],
    pub our_xpub: [u8; 32],
    shared_secret: Option<SharedSecret>,
}
// implement clone without shared_secret
impl Clone for PeerKeyPair {
    fn clone(&self) -> Self {
        PeerKeyPair {
            our_xpub: self.our_xpub.clone(),
            peer_xpub: self.peer_xpub.clone(),
            shared_secret: None,
        }
    }
}

// ignore unconditional recursion
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
    pub version: i8,
    pub ip: String,
    pub port: String,
    pub last_ping: i32,
}

impl Address {
    pub fn new(version: i8, ip: String, port: String) -> Self {
        Address {
            version,
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
    pub version: i8,
    pub hid: String,
    pub keypair: PeerKeyPair,
    pub address: Address,
}

impl Peer {
    pub fn new(version: i8, hid: String, peer_xpub: [u8; 32], address: Address) -> Self {
        let secret = StaticSecret::random_from_rng(OsRng);
        let our_xpub = PublicKey::from(&secret);
        let their_xpub = PublicKey::from(peer_xpub);
        let shared_secret = secret.diffie_hellman(&their_xpub);
        return Peer {
            version,
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
        return format!("{}:{}", self.address.ip, self.address.port);
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
    // extern crate aes_soft;
    // extern crate aes;
    // extern crate pkcs7;
    // extern crate rand;

    // use aes_soft::Aes256;
    // use aes::{block_cipher_trait::BlockCipher, NewBlockCipher};
    // use pkcs7::Padding;
    // use rand::Rng;
    // use aes::generic_array::GenericArray;

    // fn main() {
    //     let mut rng = rand::thread_rng();

    //     // This should be the shared secret derived from the Diffie-Hellman exchange.
    //     let shared_secret: [u8; 32] = [0; 32]; // TODO: Replace with actual shared secret
    //     let cipher = Aes256::new(GenericArray::from_slice(&shared_secret));

    //     let iv: [u8; 16] = rng.gen();

    //     // Data to encrypt
    //     let data = b"Hello, World!";
    //     let mut data_padded = Vec::from(data.as_ref());

    //     // Pad data to multiple of block size
    //     let padding_len = 16 - (data.len() % 16);
    //     data_padded.extend(vec![padding_len as u8; padding_len]);

    //     // Encrypt blocks
    //     let mut cipher_text = vec![0u8; data_padded.len()];
    //     for (plain_block, cipher_block) in data_padded.chunks(16).zip(cipher_text.chunks_mut(16)) {
    //         let block = GenericArray::clone_from_slice(plain_block);
    //         let cipher_block = GenericArray::from_mut_slice(cipher_block);
    //         cipher.encrypt_block(block.into(), cipher_block);
    //     }

    //     // To decrypt, create a new cipher
    //     let cipher = Aes256::new(GenericArray::from_slice(&shared_secret));

    //     // Decrypt blocks
    //     let mut decrypted_data = vec![0u8; cipher_text.len()];
    //     for (cipher_block, decrypted_block) in cipher_text.chunks(16).zip(decrypted_data.chunks_mut(16)) {
    //         let block = GenericArray::clone_from_slice(cipher_block);
    //         let decrypted_block = GenericArray::from_mut_slice(decrypted_block);
    //         cipher.decrypt_block(block.into(), decrypted_block);
    //     }

    //     // Remove padding
    //     let padding_len = *decrypted_data.last().unwrap() as usize;
    //     decrypted_data.truncate(decrypted_data.len() - padding_len);

    //     // Check that the decrypted data matches the original data
    //     assert_eq!(data, &*decrypted_data);
    // }
}
