use std::collections::{HashMap, HashSet};
use std::net::UdpSocket;
use std::sync::Arc;

use serde_json;
use tokio::io::{self, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::Mutex;
use tokio::task;

use self::types::ConduitPacket;
use crate::holon::{Address, OurPeer};

pub mod types;

// Create an alias for a channel sender that sends ConduitPackets.
pub type PacketSender = Sender<ConduitPacket>;

pub struct ConduitListener {
    our: Arc<Mutex<OurPeer>>,
    socket: UdpSocket,
    sent_packets: HashMap<u64, Vec<u8>>,
    received_packets: HashMap<u64, ConduitPacket>, // Cache of received packets.
    failed_packets: HashSet<u64>,                  // Cache of failed packets.
    window_size: u64,
    next_seq_num: u64,
    base_seq_num: u64,
    receiver: Receiver<ConduitPacket>,
}

impl ConduitListener {
    pub fn new(our_peer: &OurPeer) -> Result<(PacketSender, Self), std::io::Error> {
        let socket = UdpSocket::bind(our_peer.get_addr())?;
        let (sender, receiver) = channel(1000); // channel capacity 1000
        let our = Arc::new(Mutex::new(our_peer.clone()));

        return Ok((
            sender,
            Self {
                our: our,
                socket,
                sent_packets: HashMap::new(),
                received_packets: HashMap::new(),
                failed_packets: HashSet::new(),
                next_seq_num: 1,
                base_seq_num: 0,
                window_size: 100,
                receiver,
            },
        ));
    }

    pub async fn run(&mut self) {
        loop {
            // Listen for new packets from the channel.
            if let Some(packet) = self.receiver.recv().await {
                let mut our_peer = self.our.lock().await;

                // If the packet comes from a known peer, process it directly.
                if our_peer.peers.contains_key(&packet.from.hid) {
                    our_peer.handle_packet(packet).await.unwrap();
                } else {
                    // If the packet comes from an unknown peer, queue it.
                    // Here you can implement a queue to hold the packets from unknown peers,
                    // and decide how to handle them later.
                    // This is just a simple print statement to indicate this scenario.
                    println!("Packet from unknown peer, queued for processing later.");
                }
            }
        }
    }

    pub async fn send_packet(&self, to: String, packet: ConduitPacket) -> io::Result<()> {
        // self.our_peer.lock().await.send_packet(to, packet).await;
        // let mut stream = TcpStream::connect(destination.get_addr()).await?;
        // let packet_string = serde_json::to_string(&packet).unwrap();
        // stream.write_all(packet_string.as_bytes()).await?;

        Ok(())
    }
}

// Use this function to spawn the ConduitListener in its own task.
pub fn spawn_conduit_listener(peer: OurPeer) -> PacketSender {
    let (sender, mut our_peer) = ConduitListener::new(&peer).unwrap();
    task::spawn(async move {
        our_peer.run().await;
    });
    sender
}

#[cfg(test)]
mod tests {
    use super::*;
    use bip32::Mnemonic;

    #[tokio::test]
    async fn test_conduit_listener() {
        let mut alice = OurPeer::init(
            1,
            "Alice".into(),
            Address::new(1, "127.0.0.1".into(), 9033),
            Mnemonic::new("mention citizen dutch share final ship valid balance rack drastic mystery grief require fluid mom forget toast business snake laugh faint dentist ensure return", bip32::Language::English).unwrap(),
            "password".into(),
        );
        let alice_conduit = spawn_conduit_listener(alice);

        let mut bob = OurPeer::init(
            1,
            "Bob".into(),
            Address::new(1, "127.0.0.1".into(), 9034),
            Mnemonic::new("gas panel detail execute stairs crunch economy south truck lava mistake ladder source dry burger they barely off model abstract trim narrow they prosper", bip32::Language::English).unwrap(),
            "password".into(),
        );

        let bob_conduit = spawn_conduit_listener(bob);

        // Alice discovers Bob and sends a handshake
        // alice.create_foreign_peer(ForeignPeerEntry {
        //     epoch: 1,
        //     hid: "Bob".into(),
        //     address: Address::new(1, "127.0.0.1".into(), 9031),
        //     pubkey: bob.hd_wallet.get_networking_pubkey(),
        // });
    }
}
