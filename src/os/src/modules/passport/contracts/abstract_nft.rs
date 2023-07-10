#[derive(Clone, Debug)]
pub struct NFT {
    pub collection: String, // Milady Maker
    pub contract: String,   // 0x5Af0D9827E0c53E4799BB226655A1de152A425a5
    pub network: String,
}

impl NFT {
    pub fn describe(&self) -> String {
        format!("nft collection: {}", self.collection)
    }
}