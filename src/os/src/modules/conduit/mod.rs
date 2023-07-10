use std::collections::{HashMap, HashSet};
use std::net::UdpSocket;
use std::sync::Arc;
use std::time::Duration;

use serde_json;
use tokio::io::{self, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::Mutex;
use tokio::task;
use tokio::time::sleep;

use self::types::ConduitPacket;
use crate::holon::{Address, OurPeer};

pub mod types;

// Create an alias for a channel sender that sends ConduitPackets.
pub type PacketSender = Sender<ConduitPacket>;
pub type PacketReceiver = Receiver<ConduitPacket>;

pub struct ConduitListener {
    our: Arc<Mutex<OurPeer>>,
    socket: UdpSocket,
    sent_packets: HashMap<u64, Vec<u8>>,
    received_packets: HashMap<u64, ConduitPacket>, // Cache of received packets.
    failed_packets: HashSet<u64>,                  // Cache of failed packets.
    window_size: u64,
    next_seq_num: u64,
    base_seq_num: u64,
    receiver: PacketReceiver,
    sender: PacketSender,
}

impl ConduitListener {
    pub fn new(our_peer: &OurPeer) -> Result<ConduitListener, std::io::Error> {
        let socket = UdpSocket::bind(our_peer.get_addr())?;
        let (sender, receiver) = channel(1000); // channel capacity 1000
        let our = Arc::new(Mutex::new(our_peer.clone()));

        return Ok(ConduitListener {
            our: our,
            socket,
            sent_packets: HashMap::new(),
            received_packets: HashMap::new(),
            failed_packets: HashSet::new(),
            next_seq_num: 1,
            base_seq_num: 0,
            window_size: 100,
            receiver,
            sender,
        });
    }

    // pub async fn run(&mut self) {
    //     loop {
    //         // Listen for new packets from the channel.
    //         if let Some(packet) = self.receiver.recv().await {
    //             let mut our_peer = self.our.lock().await;

    //             // If we are sending a packet, add it to the sent packets cache.
    //             if packet.from.hid == our_peer.hid {
    //                 println!(
    //                     "{} Sending packet to: {}",
    //                     our_peer.hid,
    //                     packet.to.hid.clone()
    //                 );
    //                 self.sent_packets
    //                     .insert(packet.seq_num, packet.data.clone());

    //                 // self.send_packet(packet.clone()).await.unwrap()
    //             } else {
    //                 // If the packet comes from a known peer, process it directly.
    //                 if our_peer.peers.contains_key(&packet.from.hid) {
    //                     println!(
    //                         "{} Received packet from: {}",
    //                         our_peer.hid,
    //                         packet.from.hid.clone()
    //                     );
    //                     our_peer.handle_packet(packet).await.unwrap();
    //                 } else {
    //                     // If the packet comes from an unknown peer, verify the signature and queue it.
    //                     println!(
    //                         "{} Packet from unknown peer, queued for processing later.",
    //                         our_peer.hid.clone()
    //                     );
    //                 }
    //             }
    //         }
    //     }
    // }

    pub async fn await_packets(runtime: Arc<Mutex<ConduitListener>>) -> Result<(), &'static str> {
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
        let packet_sender = self.sender.clone();
        while let Some(packet) = self.receiver.recv().await {
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
                    // self.socket
                    //     .send_to(&seq_num.to_le_bytes(), self..addr.get_addr())
                    //     .unwrap();
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
                .send_to(&seq_num.to_le_bytes(), return_packet.to.addr.get_addr())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::holon::ForeignPeerEntry;
    use bip32::Mnemonic;

    #[tokio::test]
    async fn test_peer_handshake() {
        let mut alice = OurPeer::init(
            1,
            "Alice".into(),
            Address::new(1, "127.0.0.1".into(), 9033),
            Mnemonic::new("mention citizen dutch share final ship valid balance rack drastic mystery grief require fluid mom forget toast business snake laugh faint dentist ensure return", bip32::Language::English).unwrap(),
            "password".into(),
        );

        let mut bob = OurPeer::init(
            1,
            "Bob".into(),
            Address::new(1, "127.0.0.1".into(), 9034),
            Mnemonic::new("gas panel detail execute stairs crunch economy south truck lava mistake ladder source dry burger they barely off model abstract trim narrow they prosper", bip32::Language::English).unwrap(),
            "password".into(),
        );

        // Alice discovers Bob and sends a handshake
        alice.create_foreign_peer(ForeignPeerEntry {
            epoch: 1,
            hid: "Bob".into(),
            address: Address::new(1, "127.0.0.1".into(), 9031),
            pubkey: bob.clone().get_networking_pubkey().clone(),
        });

        // Alice discovers Bob and sends a handshake
        bob.create_foreign_peer(ForeignPeerEntry {
            epoch: 1,
            hid: "Alice".into(),
            address: Address::new(1, "127.0.0.1".into(), 9030),
            pubkey: alice.clone().get_networking_pubkey().clone(),
        });
        let alice_conduit = ConduitListener::new(&alice).unwrap();
        let bob_conduit = ConduitListener::new(&bob).unwrap();
        alice_conduit.listen();
        bob_conduit.listen();

        // let alice_conduit = spawn_conduit_listener(alice.clone());
        // let bob_conduit = spawn_conduit_listener(bob.clone());

        alice_conduit
            .send_packet(ConduitPacket {
                from: alice.clone().get_conduit_peer_entry().into(),
                to: bob.clone().get_conduit_peer_entry().into(),
                seq_num: 1,
                data: "Hello Bob!".into(),
                signature: "xyz".into(),
            })
            .await
            .unwrap();
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        // Ok(())
        // write code to delay 5 seconds
    }
}
