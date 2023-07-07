use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::net::UdpSocket;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use crate::holon::address::Address;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConduitPeer {
    pub hid: String,
    pub pubkey: [u8; 32],
    pub addr: Address,
    xpriv: Option<String>,
}

impl ConduitPeer {
    pub fn new(hid: String, addr: Address, pubkey: [u8; 32], xpriv: Option<String>) -> Self {
        ConduitPeer {
            hid,
            addr,
            pubkey,
            xpriv,
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

    fn serialize(&self) -> Result<Vec<u8>, Box<bincode::ErrorKind>> {
        bincode::serialize(self)
    }

    fn deserialize(data: &[u8]) -> Result<ConduitPacket, Box<bincode::ErrorKind>> {
        bincode::deserialize(data)
    }
}

// -----------------------------
#[derive(Debug)]
pub struct ConduitRuntime {
    our: ConduitPeer,
    peer: ConduitPeer,
    socket: UdpSocket,
    sent_packets: HashMap<u64, Vec<u8>>,
    received_packets: HashMap<u64, ConduitPacket>, // Cache of received packets.
    failed_packets: HashSet<u64>,                  // Cache of failed packets.
    window_size: u64,
    next_seq_num: u64,
    base_seq_num: u64,
    packet_sender: mpsc::Sender<ConduitPacket>,
    packet_receiver: mpsc::Receiver<ConduitPacket>,
}

impl ConduitRuntime {
    pub fn new(our: &ConduitPeer, peer: &ConduitPeer) -> Result<ConduitRuntime, std::io::Error> {
        let socket = UdpSocket::bind(our.addr.get_addr())?;
        let (tx, rx) = mpsc::channel(100);

        return Ok(ConduitRuntime {
            our: our.clone(),
            peer: peer.clone(),
            socket,
            sent_packets: HashMap::new(),
            received_packets: HashMap::new(),
            failed_packets: HashSet::new(),
            next_seq_num: 1,
            base_seq_num: 0,
            window_size: 100,
            packet_sender: tx,
            packet_receiver: rx,
        });
    }

    pub async fn await_packets(runtime: Arc<Mutex<ConduitRuntime>>) -> Result<(), &'static str> {
        loop {
            let mut runtime = runtime.lock().await;
            let packet_opt = runtime.receive_packet().await;
            if let Ok(Some(packet)) = packet_opt {
                runtime.received_packets.insert(packet.seq_num, packet);
            } else if let Err(e) = packet_opt {
                println!("Error while receiving packet: {}", e);
                break;
            }
        }
        Ok(())
    }

    pub async fn listen(mut self) -> Result<(), &'static str> {
        let packet_sender = self.packet_sender.clone();
        while let Some(packet) = self.packet_receiver.recv().await {
            self.handle_packet(packet);
        }

        tokio::spawn(async move {
            loop {
                let packet_opt = self.receive_packet().await;
                if let Ok(Some(packet)) = packet_opt {
                    if let Err(_e) = packet_sender.send(packet).await {
                        eprintln!("Failed to send packet for processing");
                    }
                } else if let Err(e) = packet_opt {
                    eprintln!("Error while receiving packet: {}", e);
                    break;
                }
            }
        });

        Ok(())
    }

    pub async fn send_packet(&mut self, packet: ConduitPacket) -> Result<(), std::io::Error> {
        let address = packet.to.addr.clone();
        // get valid address from packet addresses
        while self.next_seq_num >= self.base_seq_num + self.window_size {
            // If the window is full, wait for some ACKs to arrive.
            sleep(Duration::from_millis(100)).await;
        }

        let seq_num = self.next_seq_num;
        self.next_seq_num += 1;

        let data = packet.serialize();
        if data.is_err() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to serialize packet.",
            ));
        }
        // safely unwrap data
        let mut unwrapped_data = data.unwrap();
        unwrapped_data.splice(0..0, seq_num.to_le_bytes().to_vec());
        self.sent_packets.insert(seq_num, unwrapped_data.clone());

        self.socket
            .send_to(&unwrapped_data.clone(), address.get_addr())?;

        println!(
            "Sent packet with seq_num: {} to {} at {}",
            seq_num,
            packet.to.hid.clone(),
            address.get_addr()
        );

        Ok(())
    }

    pub async fn receive_packet(&mut self) -> Result<Option<ConduitPacket>, &'static str> {
        let mut buffer = [0u8; 4096];
        let (size, _) = self
            .socket
            .recv_from(&mut buffer)
            .map_err(|_| "Failed to receive packet")?;

        // First 8 bytes are for the sequence number
        let seq_num = u64::from_le_bytes(
            buffer[..8]
                .try_into()
                .map_err(|_| "Failed to get sequence number")?,
        );

        // Check if the received packet is in the set of sent packets (ACK)
        if self.sent_packets.contains_key(&seq_num) {
            // Packet is an ACK, remove from the set
            println!("Received ACK for packet with seq_num: {}", seq_num);
            self.sent_packets.remove(&seq_num);
            return Ok(None);
        } else {
            // Packet is new, handle accordingly
            let packet = match ConduitPacket::deserialize(&buffer[8..size]) {
                Ok(packet) => packet,
                Err(_e) => {
                    // Deserialization failed, consider it as a failed packet
                    self.failed_packets.insert(seq_num);
                    // Respond with NACK
                    self.socket
                        .send_to(&seq_num.to_le_bytes(), self.peer.addr.get_addr())
                        .unwrap();
                    return Err("Failed to deserialize data");
                }
            };

            println!(
                "Received packet with seq_num: {} from {} at {}",
                seq_num,
                packet.from.hid.clone(),
                packet.from.addr.get_addr()
            );

            let return_packet = packet.clone();
            // Check if the packet is expected, otherwise cache it
            if packet.seq_num != self.base_seq_num {
                self.received_packets.insert(packet.seq_num, packet);
            } else {
                self.handle_packet(packet);
                self.base_seq_num += 1;
                while let Some(packet) = self.received_packets.remove(&self.base_seq_num) {
                    // Handle the cached packet
                    self.handle_packet(packet);
                    self.base_seq_num += 1;
                }
            }

            // Respond with ACK
            self.socket
                .send_to(&seq_num.to_le_bytes(), self.peer.addr.get_addr())
                .map_err(|_| "Failed to ACK packet")?;
            Ok(Some(return_packet))
        }
    }

    fn handle_packet(&mut self, packet: ConduitPacket) {
        // If the sequence number is not the next expected one, cache the packet
        if packet.seq_num != self.next_seq_num {
            self.received_packets.insert(packet.seq_num, packet);
        } else {
            // Deliver the packet
            self.deliver_packet(&packet);
            self.next_seq_num += 1;

            // Check if we have subsequent packets in the cache
            while let Some(cached_packet) = self.received_packets.remove(&self.next_seq_num) {
                // Deliver the cached packet
                self.deliver_packet(&cached_packet);
                self.next_seq_num += 1;
            }
        }
    }

    pub fn deliver_packet(&mut self, packet: &ConduitPacket) {
        // Deliver the packet to the runtime
        // self.runtime.lock().unwrap().deliver_packet(packet);
        println!("Delivered packet with seq_num: {}", packet.seq_num);
    }

    pub fn resend_packet(&mut self, packet: Vec<u8>, address: &str) -> Result<(), std::io::Error> {
        self.socket.send_to(&packet, address)?;
        Ok(())
    }
}

// fn handle_nack(&self, seq_num: u64) {
//     if let Some(packet) = self.sent_packets.get(&seq_num) {
//         // If the packet is still in the sent packets set, schedule it for resending
//         let peer_addr = self.peer.get_addr().clone();
//         let packet = packet.clone();
//         let resend_future = async move {
//             sleep(Duration::from_secs(10)).await; // Wait for 10 seconds
//             self.socket.send_to(&packet, peer_addr).unwrap();
//         };
//         tokio::spawn(resend_future);
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    // #[tokio::test]
    // async fn test_new_runtime() {
    //     let address = Address::new(
    //         1,
    //         "127.0.0.1".to_string(),
    //         "8000".to_string(),
    //         Transport::Udp,
    //         0,
    //     );
    //     let peer = Peer::new(
    //         1,
    //         "hid".to_string(),
    //         "addr".to_string(),
    //         "xpub".to_string(),
    //         vec![address],
    //     );
    //     let result = ConduitRuntime::new(&peer, "127.0.0.1:8080").unwrap();
    //     assert_eq!(result.our.hid, "hid");
    // }

    // #[tokio::test]
    // async fn test_send_packet() {
    //     let address = Address::new(
    //         1,
    //         "127.0.0.1".to_string(),
    //         "8000".to_string(),
    //         Transport::Udp,
    //         0,
    //     );
    //     let peer = Peer::new(
    //         1,
    //         "hid".to_string(),
    //         "addr".to_string(),
    //         "xpub".to_string(),
    //         vec![address],
    //     );
    //     let mut runtime = ConduitRuntime::new(&peer, "127.0.0.1:8081").unwrap();
    //     let packet = ConduitPacket::new(0, vec![1, 2, 3], &peer, vec![1, 2, 3]);
    //     let result = runtime.send_packet(packet, "127.0.0.1:8082").await;
    //     assert!(result.is_ok());
    // }

    // #[tokio::test]
    // async fn test_receive_packet() {
    //     let address = Address::new(
    //         1,
    //         "127.0.0.1".to_string(),
    //         "8000".to_string(),
    //         Transport::Udp,
    //         0,
    //     );
    //     let peer = Peer::new(
    //         1,
    //         "hid".to_string(),
    //         "addr".to_string(),
    //         "xpub".to_string(),
    //         vec![address],
    //     );
    //     let mut runtime = ConduitRuntime::new(&peer, "127.0.0.1:8083").unwrap();
    //     let result = runtime.receive_packet();
    //     assert!(result.is_ok() || result.is_err());
    // }
}

// use aes_gcm::Aes256Gcm; // Or Aes128Gcm
// use aes_gcm::aead::{Aead, NewAead, generic_array::GenericArray};
// use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};

// use rand_core::OsRng;

// // Your server private key
// let server_private_key = EphemeralSecret::new(OsRng);

// // Get peer's public key from xpub string
// let peer_public_key = PublicKey::from(/* convert xpub string to byte array */);

// // Derive the shared secret
// let shared_secret: SharedSecret = server_private_key.diffie_hellman(&peer_public_key);

// // Create a 256-bit key for AES from the shared secret
// let key = GenericArray::from_slice(shared_secret.as_bytes()); // if AES256

// // Create a new AES instance
// let cipher = Aes256Gcm::new(key);

// // This nonce should be unique for every message
// let nonce = GenericArray::from_slice(/*unique nonce bytes*/);

// // Let's say we have some data
// let plaintext = b"plaintext data";

// // Encrypt the data
// let ciphertext = cipher.encrypt(nonce, plaintext.as_ref()).expect("encryption failure!");

// // Decrypt the data
// let decrypted_data = cipher.decrypt(nonce, ciphertext.as_ref()).expect("decryption failure!");
