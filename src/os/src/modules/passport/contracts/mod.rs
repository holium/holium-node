pub mod abstract_nft;
pub mod azimuth;
pub mod ens;

#[derive(Clone, Debug)]
pub enum IdentitySystem {
    Azimuth(azimuth::Azimuth),
    ENS(ens::ENS),
    NFT(abstract_nft::NFT),
}

impl IdentitySystem {
    pub fn describe(&self) -> String {
        match self {
            IdentitySystem::Azimuth(az) => az.describe(),
            IdentitySystem::ENS(ens) => ens.describe(),
            IdentitySystem::NFT(nft) => nft.describe(),
        }
    }
}
