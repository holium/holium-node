use std::vec;

use async_trait::async_trait;

use crate::holon::{Address, OurPeer};

use crate::action::{Action, PrebootArgs};
use crate::effect::Effect;
use crate::global_state::{GlobalStateChange, PassportSetData};
use crate::states::booting::Booting;
use crate::states::{CloneState, State};
use crate::OS;

use super::{ActionResult, StateBox};

#[derive(Clone)]
pub struct Genesis;

/// # Genesis
/// Genesis is the first state of the system. It is responsible for fetching identity data
/// from an eth node and verifying the eth addr with the identity system public key.
impl Genesis {
    pub fn new() -> Genesis {
        Genesis
    }

    pub async fn start_preboot(&self) {
        println!(
            "{}:preboot - creating directories and files",
            self.describe()
        );
        println!("{}:preboot - fetching azimuth state", self.describe());
        // self.get_azimuth_state().await.unwrap();
        println!("{}:preboot - checking for updates", self.describe());
    }

    pub fn create_passport(&self, args: PrebootArgs) -> Effect {
        println!(
            "{}:preboot - creating passport for {}",
            self.describe(),
            args.hid
        );

        let (protocol, addr, port) = parse_address(&args.bind_address).unwrap();
        let address = Address::new(protocol, addr, port);
        println!(
            "{}:preboot - creating HD wallet {}",
            self.describe(),
            args.hid
        );

        let identity = OurPeer::from_seed(1, args.hid, address, args.boot_key);

        Effect::GlobalStateChange(GlobalStateChange::PassportCreated(PassportSetData {
            identity: identity,
            identity_system: args.identity_system,
        }))
    }

    // pub async fn get_azimuth_state(&self) -> Result<(), Box<dyn std::error::Error>> {
    //     // 1. Fetch azimuth snapshot from eth node
    //     let transport = web3::transports::Http::new("http://localhost:8545")?;
    //     let web3 = web3::Web3::new(transport);

    //     let contract_address: EthAddr =
    //         EthAddr::from_str("0x223c067f8cf28ae173ee5cafea60ca44c335fecb").unwrap();

    //     let contract = Contract::from_json(
    //         web3.eth(),
    //         contract_address,
    //         include_bytes!("../modules/passport/contracts/abi/azimuth_abi.json"),
    //     )?;

    //     let zod = EthAddr::from_str("0x9F57C77b1095BD5Db0558b9Cb9b8e6Fc67375E3C").unwrap();

    //     let result: bool = contract
    //         .query("isOwner", (0, zod), None, Options::default(), None)
    //         .await?;

    //     println!("Query result: {:?}", result);
    //     // let result = contract.query("getKeys", (), None, Options::default(), None);
    //     // let public_key: U256 = result.await.unwrap();

    //     // println!("Public Key: {:?}", public_key);

    //     // let message = "message to sign"; // replace with your actual message
    //     // let signature = "signature_here"; // replace with the actual signature

    //     // let secp = Secp256k1::new();
    //     // let message = Message::from_slice(&message.as_bytes()).unwrap();
    //     // let signature = Signature::from_compact(&Vec::from_hex(signature).unwrap()).unwrap();
    //     // let recovery_id = RecoveryId::from_i32(0).unwrap(); // You need to know the recovery id
    //     // let public_key_secp = secp.recover(&message, &signature, &recovery_id).unwrap();

    //     // println!("Recovered Public Key: {:?}", public_key_secp);

    //     Ok(())
    // }
}

#[async_trait(?Send)]
impl State for Genesis {
    fn describe(&self) -> String {
        String::from("genesis")
    }
    async fn perform_action(&self, action: Action, _os_state: &OS) -> ActionResult {
        match action {
            Action::Preboot(preboot_args) => {
                self.start_preboot().await;
                let passport_effect = self.create_passport(preboot_args);

                ActionResult::Ok(vec![
                    passport_effect,
                    Effect::StateTransition(Box::new(Booting::new())),
                ])
            }
            _ => ActionResult::Ok(vec![]),
        }
    }
}

impl CloneState for Genesis {
    fn clone_box(&self) -> StateBox {
        Box::new(self.clone())
    }
}

/// parse_address: parses a string of the form /1/
/// TODO move to helper module
///
/// Example
/// ```
/// let (protocol, addr, port) = parse_address("/1/
/// ```
fn parse_address(addr: &str) -> Result<(i8, String, i16), &str> {
    let mut parts = addr.split('/');
    if parts.next() != Some("") {
        return Err("Invalid format");
    }

    // cast protocol to number
    let protocol = match parts.next() {
        Some(p) => p.parse::<i8>().unwrap(),
        None => return Err("Protocol not found"),
    };

    let addr_port = match parts.next() {
        Some(ap) => ap,
        None => return Err("Address and port not found"),
    };

    let mut addr_port_parts = addr_port.split(':');
    let addr = match addr_port_parts.next() {
        Some(a) => a.to_string(),
        None => return Err("Address not found"),
    };
    let port = match addr_port_parts.next() {
        Some(p) => match p.parse::<i16>() {
            Ok(port) => port,
            Err(_) => return Err("Invalid port number"),
        },
        None => return Err("Port not found"),
    };

    Ok((protocol, addr, port))
}
