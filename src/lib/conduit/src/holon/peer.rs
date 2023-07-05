use std::{collections::HashMap, hash::Hash, rc::Rc};

use bip32::Mnemonic;

use serde::{Deserialize, Serialize};
use x25519_dalek::{PublicKey, SharedSecret, StaticSecret};

use super::{
    address::Address,
    hd_wallet::HDWallet,
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

// #[derive(Clone, Serialize, Deserialize)]
pub struct ForeignPeer {
    pub epoch: i8, // the epoch of the peer, should be incremented when the keypair is updated
    pub hid: String, // the holon id of the peer
    pub address: Address, // the ip address and port of the peer
    pub pubkey: [u8; 32], // the public key of the peer
    shared_secret: Option<SharedSecret>,
    state: ForeignPeerState,
    state_listener: Rc<dyn StateChangeListener>,
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

#[derive(Clone, Serialize, Deserialize)]
pub struct OurPeer {
    pub epoch: i8, // the epoch of the peer, should be incremented when the keypair is updated
    pub hid: String,
    pub address: Address, // the ip address and port of the peer
    hd_wallet: HDWallet,
    peers: HashMap<String, ForeignPeer>,
}

impl OurPeer {
    pub fn init(
        epoch: i8,
        hid: String,
        address: Address,
        mnemonic_phrase: Mnemonic,
        password: String,
    ) -> Self {
        let hd_wallet = HDWallet::new(mnemonic_phrase.to_seed(password.as_str()).as_bytes());
        return OurPeer {
            epoch,
            hid,
            address,
            hd_wallet,
            peers: HashMap::new(),
        };
    }

    pub fn get_addr(&self) -> String {
        self.address.get_addr()
    }

    pub fn sign_message(&self, message: &[u8]) -> [u8; 64] {
        self.hd_wallet.sign_message(message)
    }

    pub fn add_foreign_peer(&mut self, peer: ForeignPeer) {
        self.peers.insert(peer.hid.clone(), peer);
    }

    // pub fn discover_peer

    // Transition from HandshakeReceived to Verified or Rejected
    pub fn process_handshake(
        &mut self,
        hid: String,
        pubkey: Vec<u8>,
        signature: Vec<u8>,
    ) -> Result<(), &'static str> {
        if let Some(peer) = self.peers.get_mut(&hid) {
            match peer.state {
                ForeignPeerState::HandshakeSent => {
                    // Verify the signature
                    // if !verify_signature(&pubkey, &signature) {
                    //    peer.state = PeerState::Rejected;
                    //    return Err("Invalid signature");
                    // }

                    // let pubkey_array = <[u8; 32]>::try_from(&pubkey[..]).unwrap();
                    // *peer = ForeignPeerState::new(
                    //     1,
                    //     "hid".into(),
                    //     Address::new(1, "127.0.0.1".into(), 9030),
                    //     pubkey_array,
                    //     self.hd_wallet.private_key,
                    // );
                    peer.state = ForeignPeerState::Verified;
                    Ok(())
                }
                _ => Err("Invalid state transition"),
            }
        } else {
            Err("Peer not found")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gen_test_peer() -> OurPeer {
        let epoch = 1;
        let hid = "test".to_string();
        let address = Address::new(1, "127.0.0.1".to_string(), 9030);
        let mnemonic_phrase = Mnemonic::new("mention citizen dutch share final ship valid balance rack drastic mystery grief require fluid mom forget toast business snake laugh faint dentist ensure return", bip32::Language::English).unwrap();
        let password = "password".to_string();
        let our_peer = OurPeer::init(epoch, hid, address, mnemonic_phrase, password);
        our_peer
    }

    fn gen_test_foreign_peer(our_privkey: [u8; 32]) -> ForeignPeer {
        let epoch = 1;
        let hid = "test".to_string();
        let address = Address::new(1, "127.0.0.1".to_string(), 9031);
        let peer_pubkey = [0u8; 32];
        let foreign_peer = ForeignPeer::new(epoch, hid, address, peer_pubkey, our_privkey);
    }

    #[test]
    fn init_our_peer() {
        let our_peer = gen_test_peer();
        assert_eq!(our_peer.epoch, 1);
    }

    #[test]
    fn it_discovers_foreign_peer() {
        let our_peer = gen_test_peer();
    }
}
