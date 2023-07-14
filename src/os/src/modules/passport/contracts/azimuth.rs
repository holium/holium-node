use ethers::{abi::Abi, prelude::*, abi::FixedBytes};
use std::sync::Arc;

// statically generate all the Azimuth contract methods and types from the ABI
abigen!(AziAbi, "./src/os/src/modules/passport/contracts/abi/azimuth.json", event_derives(serde::Deserialize, serde::Serialize));

#[derive(Clone, Debug)]
pub struct Azimuth {
    pub contract_address: String,
    pub network: String,
}

pub type RawAzimuthPoint = ([u8; 32], [u8; 32], bool, bool, bool, u32, u32, u32, u32, u32);
#[derive(Debug)]
pub struct AzimuthPoint {
    point: u32,
    encryption_key: [u8; 32],
    authentication_key: [u8; 32],
    has_sponsor: bool,
    active: bool,
    escape_requested: bool,
    sponsor: u32,
    escape_requested_to: u32,
    crypto_suite_version: u32,
    key_revision_number: u32,
    continuity_number: u32,
}
pub type AllAzimuthPoints = Vec<AzimuthPoint>;

impl Azimuth {
    pub fn new(contract_address: Option<String>, network: String) -> Self {
        match contract_address {
            Some(s) => Self { contract_address: s, network },
            // fallback to the eth mainnet contract address when it's not passed in
            None =>  Self { contract_address: String::from("0x223c067f8cf28ae173ee5cafea60ca44c335fecb") , network },
        }
    }

    pub fn describe(&self) -> String {
        "azimuth".to_string()
    }

    // returns a vector of AzimuthPoint structs based on the Activated events in the blockchain's history
    pub async fn get_full_state(&self) -> Result<AllAzimuthPoints, Box<dyn std::error::Error>> {
        // setup api interface
        let provider = Provider::<Http>::try_from("http://localhost:8545")?;
        if self.network == "mainnet" {
            // TODO use an actual mainnet provider
            let provider = Provider::<Http>::try_from("http://localhost:8545")?;
        }
        let client = Arc::new(provider);
        let contract_addr = self.contract_address.parse::<Address>()?;
        let contract = AziAbi::new(contract_addr.clone(), client.clone());

        // query for all activated points
        let logs: Vec<ActivatedFilter> = contract
            .activated_filter()
            .from_block(0)
            .query()
            .await?;

        let mut result: AllAzimuthPoints = Vec::new();
        for p in &logs {
            println!("point === {:#?}", p.point);
            let point: RawAzimuthPoint = contract.points(p.point).call().await?;
            result.push(AzimuthPoint {
                point: p.point,
                encryption_key: point.0,
                authentication_key: point.1,
                has_sponsor: point.2,
                active: point.3,
                escape_requested: point.4,
                sponsor: point.5,
                escape_requested_to: point.6,
                crypto_suite_version: point.7,
                key_revision_number: point.8,
                continuity_number: point.9,
            });
        }

        println!("{:#?}", result);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_azimuth_snapshot() {
        let azimuth = Azimuth::new(
            // localfakenet contract address
            Some("0x863d9c2e5c4c133596cfac29d55255f0d0f86381".to_string()),
            "mainnet".to_string(),
        );
        azimuth.get_full_state().await.unwrap();
    }
}
