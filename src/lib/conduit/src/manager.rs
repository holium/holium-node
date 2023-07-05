use crate::conduit::ConduitPacket;
use crate::conduit::ConduitPeer;
use crate::conduit::ConduitRuntime;
use std::sync::Arc;
use tokio::sync::Mutex;

use std::collections::HashMap;

pub type PeerId = String;
pub struct ConduitManager {
    our: ConduitPeer,
    conduits: HashMap<PeerId, (ConduitPeer, Arc<Mutex<ConduitRuntime>>)>,
}

impl ConduitManager {
    pub fn new(our: ConduitPeer) -> Self {
        ConduitManager {
            our,
            conduits: HashMap::new(),
        }
    }

    pub fn add_peer(&mut self, peer: &ConduitPeer) {
        let runtime = ConduitRuntime::new(&self.our, &peer).unwrap();
        let peer_runtime = Arc::new(Mutex::new(runtime));

        self.conduits
            .insert(peer.hid.clone(), (peer.clone(), peer_runtime.clone()));
        // peer_runtime.lock().await.send_handshake().await.unwrap();
        // ConduitManager::listen(peer_runtime.clone())
    }
    // spawn listen task
    pub fn listen(runtime: Arc<Mutex<ConduitRuntime>>) {
        tokio::spawn(async move {
            if let Err(e) = ConduitRuntime::await_packets(runtime).await {
                println!("Error while listening: {}", e);
            }
        });
    }

    pub async fn send_data_to_peer(
        &self,
        data: Vec<u8>,
        seq_num: u64,
        peer: PeerId,
    ) -> Result<(), std::io::Error> {
        let shared_secret = vec![]; // You should replace this with your actual shared_secret

        if let Some((conduit_peer, conduit_runtime)) = self.conduits.get(&peer) {
            let packet = ConduitPacket::new(&self.our, conduit_peer, seq_num, data, shared_secret);
            let mut conduit = conduit_runtime.lock().await;
            conduit.send_packet(packet.clone()).await.unwrap();
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No conduit found for the specified peer",
            ))
        }
    }
}

// #[cfg(test)]
// mod tests {

//     use crate::conduit::{ConduitPacket, ConduitPeer, ConduitRuntime};
//     use crate::manager::ConduitManager;
//     use std::process;
//     use std::sync::Arc;
//     use tokio::sync::Mutex;

//     use crate::holon::Address;
//     use crate::holon::Peer;
//     use tokio::test;

//     #[tokio::test]
//     async fn test_conduit_manager() {
//         let mut manager1 = ConduitManager::new(ConduitPeer::new(
//             "~lomder-librun".to_string(),
//             Address::new(1, "127.0.0.1".to_string(), "9030".to_string()),
//             "6E89sN2ofDQbsxco5XX6gEsf7V7gPcJkGBost2N5RVUT1YtLzdCUFPkBWkdmwdFBcxe4papfCDUX4Tv9p92MoYBbUw3gRFxwjZMppCKUgqM".to_string(),
//             Some("9s21ZrQH143K2Csy6hcjxCFeEK2S4qLo8zGSP5P5LoQmvURahfjHTurgbdzd6BKNFNTXskvrgfxZoS3KNqTYvsuRpSvQQ3VWqzE5LkQEEhS".to_string())
//         ));

//         let peer = ConduitPeer::new(
//             "~fasnut-famden".to_string(),
//             Address::new(1, "127.0.0.1".to_string(), "9031".to_string()),
//             "9s21ZrQH143K44Zx4RPNKEdidAjU4wfxg9Q1wrac19hu7pjc2UTHABFZtyTcXLE7Zvry7tQWrtpJqpRESqx7w9P4AjUy2vfFyFx7dEQ4GGX".to_string(),
//             Some("6Dky7BzBpVHUNS2YwdFMsJYFyAREivrXUmX3tC8mG7rM2Edkg3MyoVeBr9yKY65SLugNuvWG4q8ki9dPq8qfCGuksbWxM4nnRhTvVYDrcbK".to_string())
//         );

//         let peer1_id = manager1.our.hid.clone();
//         let peer2_id = peer.hid.clone();

//         let mut manager2 = ConduitManager::new(peer);

//         manager1.add_peer(&manager2.our.clone());
//         manager2.add_peer(&manager1.our);

//         // get manager1's peer
//         // let (_, peer1_runtime) = manager1.conduits.get(&peer2_id).unwrap();
//         // let runtime_clone_1 = Arc::clone(&peer1_runtime);
//         // tokio::spawn(async move {
//         //     ConduitManager::listen(runtime_clone_1.clone());
//         // });

//         // get manager2's peer
//         let (_, peer2_runtime) = manager2.conduits.get(&peer1_id).unwrap();
//         let runtime_clone_2 = Arc::clone(&peer2_runtime);
//         tokio::spawn(async move {
//             runtime_clone_2.lock().await;
//         });

//         manager1
//             .send_data_to_peer("hello".as_bytes().to_ascii_lowercase(), 1, peer2_id.clone())
//             .await
//             .unwrap();

//         manager1
//             .send_data_to_peer(
//                 "hello2".as_bytes().to_ascii_lowercase(),
//                 1,
//                 peer2_id.clone(),
//             )
//             .await
//             .unwrap();

//         tokio::time::sleep(std::time::Duration::from_secs(5)).await; // wait for 5 seconds before the test finishes
//     }
// }
