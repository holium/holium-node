use ethers::{abi::Abi, prelude::*, abi::FixedBytes};
use std::sync::Arc;
use std::io::{self, BufReader};
use std::io::prelude::*;
use std::fs::File;
use std::error::Error;

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

    pub async fn get_point(&self, point: u32) -> Result<AzimuthPoint, Box<dyn Error>> {
        let provider = Provider::<Http>::try_from("http://localhost:8545")?;
        if self.network == "mainnet" {
            // TODO use an actual mainnet provider
            let provider = Provider::<Http>::try_from("http://localhost:8545")?;
        }
        let client = Arc::new(provider);
        let contract_addr = self.contract_address.parse::<Address>()?;
        let contract = AziAbi::new(contract_addr.clone(), client.clone());
        let rpoint: RawAzimuthPoint = contract.points(point).call().await?;
        Ok(AzimuthPoint {
            point: point,
            encryption_key: rpoint.0,
            authentication_key: rpoint.1,
            has_sponsor: rpoint.2,
            active: rpoint.3,
            escape_requested: rpoint.4,
            sponsor: rpoint.5,
            escape_requested_to: rpoint.6,
            crypto_suite_version: rpoint.7,
            key_revision_number: rpoint.8,
            continuity_number: rpoint.9,
        })
    }
    // returns a vector of AzimuthPoint structs based on the Activated events in the blockchain's history
    pub async fn get_full_state(&self) -> Result<AllAzimuthPoints, Box<dyn Error>> {
        // setup api interface
        let provider = Provider::<Http>::try_from("http://localhost:8545")?;
        if self.network == "mainnet" {
            // TODO use an actual mainnet provider
            let provider = Provider::<Http>::try_from("http://localhost:8545")?;
        }
        let client = Arc::new(provider);
        let contract_addr = self.contract_address.parse::<Address>()?;
        let contract = AziAbi::new(contract_addr.clone(), client.clone());

        // our result container
        let mut result: AllAzimuthPoints = Vec::with_capacity(2000);
        let mut known_points: Vec<u32> = Vec::with_capacity(20000);

        // first, get all the points we hardcoded knowledge of
        let f = File::open("points.txt")?;
        let fileReader = BufReader::new(f);
        for line in fileReader.lines() {
            let string = line.unwrap();
            let point: u32 = string.trim().parse().expect("not a number");
            known_points.push(point);
/*
            let rpoint: RawAzimuthPoint = contract.points(point).call().await?;
            result.push(AzimuthPoint {
                point: point,
                encryption_key: rpoint.0,
                authentication_key: rpoint.1,
                has_sponsor: rpoint.2,
                active: rpoint.3,
                escape_requested: rpoint.4,
                sponsor: rpoint.5,
                escape_requested_to: rpoint.6,
                crypto_suite_version: rpoint.7,
                key_revision_number: rpoint.8,
                continuity_number: rpoint.9,
            });
*/
        }
        // 17,715,266
        // query for all activated points
//        for n in 678500..1771546 {
        for n in 680000..700000 {
            let logs: Vec<ActivatedFilter> = contract
                .activated_filter()
                .from_block(n * 10)
                .to_block((n+1) * 10)
                .query()
                .await?;
            if logs.len() == 0 { continue; }

            println!("from block {}", n * 10);

            for l in &logs {
                if l.point > 287535 {
                    println!("point === {}", l.point);
                    known_points.push(l.point);
                }
            }
        }

/*
        for p in &all_points {
            println!("point === {}", *p);
            let point: RawAzimuthPoint = contract.points(*p).call().await?;
            result.push(AzimuthPoint {
                point: *p,
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
        for p in 287535..4_000_000_000 {
            let point = self.get_point(p).await?;
            if point.active {
                println!("{}", p);
                result.push(point)
            }
        }
        */

        println!("result.lent {:#?}", result.len());
        println!("result[0] {:#?}", &result[0]);
        Ok(result)
    }

    // returns a tuple of (list of activated points, last block # checked)
    pub async fn get_points_from_blocks(&self, from_block: u32, to_block: u32) -> Result<(Vec<u32>, u32), Box<dyn Error>> {
        // setup api interface
        let provider = Provider::<Http>::try_from("http://localhost:8545")?;
        if self.network == "mainnet" {
            // TODO use an actual mainnet provider
            let provider = Provider::<Http>::try_from("http://localhost:8545")?;
        }
        let client = Arc::new(provider);
        let contract_addr = self.contract_address.parse::<Address>()?;
        let contract = AziAbi::new(contract_addr.clone(), client.clone());

        // our result container
        let mut points: Vec<u32> = Vec::with_capacity(2*2*2*2*2*2*2*2*2*2*2*2);

        // 17,715,266
        // query for all activated points
        for n in from_block..to_block {
            let logs: Vec<ActivatedFilter> = contract
                .activated_filter()
                .from_block(n * 10)
                .to_block((n+1) * 10)
                .query()
                .await?;
            println!("block checked {}0", n + 1);
            if logs.len() == 0 { continue; }

            for l in &logs {
                if l.point > 287535 {
                    println!("point {}", l.point);
                    points.push(l.point);
                }
            }
        }

        println!("last block checked {}0", to_block + 1);

        Ok((points, (to_block + 1) *10))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_azimuth_snapshot() {
        let azimuth = Azimuth::new(
            // localfakenet contract address
            //Some("0x863d9c2e5c4c133596cfac29d55255f0d0f86381".to_string()),
            None,
            "mainnet".to_string(),
        );
        //azimuth.get_full_state().await.unwrap();
        azimuth.get_points_from_blocks(760_520,800_000).await.unwrap();
    }
}
