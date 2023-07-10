use ethers::{abi::Abi, prelude::*};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Azimuth {
    pub contract: String,
    pub network: String,
}

impl Azimuth {
    pub fn new(contract: String, network: String) -> Self {
        Self { contract, network }
    }

    pub fn describe(&self) -> String {
        "azimuth".to_string()
    }

    pub async fn get_full_state(&self) -> Result<(), Box<dyn std::error::Error>> {
        let provider = Provider::<Http>::try_from("http://localhost:8545")?;
        let client = Arc::new(provider);

        let contract_address = "0x223c067f8cf28ae173ee5cafea60ca44c335fecb".parse::<Address>()?;
        let abi: Abi = serde_json::from_str(include_str!("./abi/azimuth.json")).unwrap();

        let contract = Contract::new(contract_address, abi, client);

        let zod = "0x9F57C77b1095BD5Db0558b9Cb9b8e6Fc67375E3C".parse::<Address>()?;

        println!("{:?}", contract.abi().functions_by_name("isOwner"));

        // let result = contract.decode_output_raw(name, bytes)
        //     .method::<_, bool>("isOwner", (0, zod))
        //     .unwrap()
        //     .call()
        //     .await
        //     .unwrap();

        let result: (bool,) = contract
            .method::<_, (bool,)>("isOwner", (0, zod))?
            .call()
            .await?;

        println!("Query result: {:?}", result.0);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_azimuth_snapshot() {
        let azimuth = Azimuth::new(
            "0x223c067f8cf28ae173ee5cafea60ca44c335fecb".to_string(),
            "mainnet".to_string(),
        );
        azimuth.get_full_state().await.unwrap();
    }
}
