// crate imports
use bip32::Mnemonic;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::conduit::ConduitPacket;

// project imports
use super::{
    address::Address, hd_wallet::HDWallet, ForeignPeer, ForeignPeerEntry, ForeignPeerState,
};

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
            peers: HashMap::with_capacity(1000), // research reallocation if capacity is reached
        };
    }

    pub fn get_addr(&self) -> String {
        self.address.get_addr()
    }

    pub fn sign_message(&self, message: &[u8]) -> [u8; 64] {
        self.hd_wallet.sign_message(message)
    }

    pub fn create_foreign_peer(&mut self, peer: ForeignPeerEntry) {
        // Create ForeignPeer from ForeignPeerEntry
        let new_foreign_peer = ForeignPeer::new(peer.epoch, peer.hid, peer.address, peer.pubkey);
        self.peers
            .insert(new_foreign_peer.hid.clone(), new_foreign_peer);
    }

    // -----------------------------------------
    // ----------- State transitions -----------
    // -----------------------------------------
    pub async fn handle_packet(&mut self, packet: ConduitPacket) -> Result<(), &'static str> {
        // Extract the sender's hid from the packet
        let hid = packet.from.hid.clone();
        let foreign_peer = self.peers.get(&hid);

        // Look up the foreign peer using the hid
        match foreign_peer {
            Some(peer) => {
                // Handle the packet based on the peer's state
                match peer.state {
                    ForeignPeerState::HandshakeReceived => self.process_handshake(
                        peer.hid.clone(),
                        peer.pubkey,
                        packet.signature.clone(),
                    ),
                    // ... handle other states ...
                    _ => Err("Invalid state transition"),
                }
            }
            None => {
                // If the peer is not found, validate the signature
                if self.validate_signature(&packet) {
                    // Create a new ForeignPeer and add to the peers HashMap
                    let new_peer = ForeignPeer::new(
                        1,
                        packet.from.hid.clone(),
                        packet.from.addr,
                        packet.from.pubkey,
                    );
                    self.peers.insert(new_peer.hid.clone(), new_peer);
                    Ok(())
                } else {
                    Err("Invalid Signature")
                }
            }
        }
    }

    pub fn validate_signature(&self, packet: &ConduitPacket) -> bool {
        // Implement your signature validation logic here. This is a placeholder.
        // Make sure to import and use proper cryptographic libraries and methodologies.
        println!(
            "validate_signature - pubkey: {:?}, signature: {:?}",
            packet.from.pubkey, packet.signature
        );
        true
    }

    // Transition from Discovered to HandshakeSent
    pub fn send_handshake(&mut self, hid: String) -> Result<(), &'static str> {
        if let Some(peer) = self.peers.get_mut(&hid) {
            match peer.state {
                ForeignPeerState::Discovered => {
                    // Simulate sending handshake
                    println!("Sending handshake to peer {}", hid);
                    peer.state = ForeignPeerState::HandshakeSent;
                    Ok(())
                }
                _ => Err("Invalid state transition"),
            }
        } else {
            Err("Peer not found")
        }
    }

    // Transition from HandshakeSent to HandshakeReceived
    pub fn receive_handshake(&mut self, hid: String) -> Result<(), &'static str> {
        if let Some(peer) = self.peers.get_mut(&hid) {
            match peer.state {
                ForeignPeerState::HandshakeSent => {
                    // Simulate receiving handshake
                    println!("Received handshake from peer {}", hid);
                    peer.state = ForeignPeerState::HandshakeReceived;
                    Ok(())
                }
                _ => Err("Invalid state transition"),
            }
        } else {
            Err("Peer not found")
        }
    }

    // Transition from HandshakeReceived to Verified or Rejected
    pub fn process_handshake(
        &mut self,
        hid: String,
        pubkey: [u8; 32],
        signature: String,
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

    fn gen_foreign_peer_entry() -> ForeignPeerEntry {
        let mnemomic = Mnemonic::new("gas panel detail execute stairs crunch economy south truck lava mistake ladder source dry burger they barely off model abstract trim narrow they prosper", bip32::Language::English).unwrap();
        let password = "pass";

        let hd_wallet = HDWallet::new(mnemomic.to_seed(password).as_bytes());

        ForeignPeerEntry {
            epoch: 1,
            hid: "~fasnut-famden".to_string(),
            address: Address {
                proto_version: 1,
                ip: "127.0.0.1".to_string(),
                port: 9031,
                last_ping: 0,
            },
            pubkey: hd_wallet.get_networking_pubkey(),
        }
    }

    fn gen_test_peer() -> OurPeer {
        let epoch = 1;
        let hid = "test".to_string();
        let address = Address::new(1, "127.0.0.1".to_string(), 9030);
        let mnemonic_phrase = Mnemonic::new("mention citizen dutch share final ship valid balance rack drastic mystery grief require fluid mom forget toast business snake laugh faint dentist ensure return", bip32::Language::English).unwrap();
        let password = "password".to_string();
        let our_peer = OurPeer::init(epoch, hid, address, mnemonic_phrase, password);
        our_peer
    }

    #[test]
    fn init_our_peer() {
        let our_peer = gen_test_peer();
        assert_eq!(our_peer.epoch, 1);
    }

    #[test]
    fn test_discovery_transition() {
        let mut our_peer = gen_test_peer();
        let foreign_peer_entry = gen_foreign_peer_entry();
        our_peer.create_foreign_peer(foreign_peer_entry.into());

        println!(
            "length: {:?}, capacity: {:?}",
            our_peer.peers.len(),
            our_peer.peers.capacity()
        )
    }
}
