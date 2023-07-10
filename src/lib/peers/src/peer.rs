extern crate ethers;

use std::{fs::File, io::prelude::*};

use bip32::{Error, Mnemonic, Prefix, XPrv};
use coins_bip39::English;
use ethers::{prelude::rand, signers::MnemonicBuilder};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Address {
    pub version: i8,
    pub derivation_path: String,
    pub addr: String,
    pub xpub: String,
    xprv: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Peer {
    pub version: i8,
    pub hid: String,
    pub xpub: String,
    xprv: String,
    root_xprv: String,
    mnemonic: String,
}

impl Peer {
    /// Returns a newly created peer and stores the keyfiles
    /// in a directory.
    ///
    /// # Arguments
    ///
    /// * `identity_id` - The name of the identity record
    /// * `password` - A password used to generate your keys.
    ///
    /// # Examples
    ///
    /// ```
    /// use passport::crypto::Wallet;
    /// let wallet = Wallet::new("lucid-mysterium".to_string(), &"password123".to_string());
    /// ```
    pub fn new(hid: String, password: &String) -> (Peer, String) {
        let mnemonic = Peer::get_words();
        let mnemonic_str = mnemonic.phrase().to_string();

        return (
            Peer::from_phrase(hid, mnemonic, password).unwrap(),
            mnemonic_str,
        );
    }

    pub fn new_eth() -> Result<(), Error> {
        let mut rng = rand::thread_rng();
        let wallet = MnemonicBuilder::<English>::default()
            .word_count(24)
            .build_random(&mut rng);

        let seed = wallet.unwrap();
        print!("seed: {:?}", seed);
        Ok(())
    }

    /// Get mnemonic words
    pub fn get_words() -> Mnemonic {
        let mnemonic = Mnemonic::random(&mut OsRng, Default::default());
        return mnemonic;
    }

    // TODO remove this after testing
    fn _write_keyfile(&self, _xprv: String) -> std::io::Result<()> {
        let mut file = File::create(format!("{}.key", self.hid))?;
        file.write_all(self.xprv.as_bytes())?;
        Ok(())
    }

    pub fn from_phrase(hid: String, phrase: Mnemonic, password: &String) -> Result<Peer, Error> {
        let seed = phrase.to_seed(password);
        // println!("seed: {:?}", seed.as_bytes());
        let root_xprv = XPrv::new(&seed)?; // root private key
                                           // TODO in this function we generate all children keys
        let child_path = "m/44'/60'/0'/0";
        let child_xprv = XPrv::derive_from_path(&seed, &child_path.parse()?)?;
        let child_xpub = child_xprv.public_key();
        let child_xprv_str = child_xprv.to_string(Prefix::XPRV);
        let child_xpub_str = child_xpub.to_string(Prefix::XPUB);

        // print!("child_xprv: {:?}", child_xprv_str);
        // print!("child_xpub: {:?}", child_xpub_str);

        return Ok(Self {
            version: 1,
            hid: hid.to_string(),
            xpub: child_xpub_str,
            xprv: child_xprv_str.to_string(),
            root_xprv: root_xprv.to_string(Prefix::XPRV).to_string(),
            mnemonic: phrase.phrase().to_string(),
        });
    }
    // pub fn from_phrase(hid: String, phrase: Mnemonic, password: &String) -> Result<Peer, Error> {
    //     let seed = phrase.to_seed(password);

    //     // Generate a hash from the seed
    //     let mut hasher = Sha256::new();
    //     hasher.update(&seed);
    //     let seed_hash = hasher.finalize();

    //     // Create the secret key from the hash
    //     let mut secret_key_bytes = [0u8; 32];
    //     secret_key_bytes.copy_from_slice(&seed_hash[..32]);
    //     let secret_key = StaticSecret::from(secret_key_bytes);

    //     // Derive the public key from the secret key
    //     let public_key = PublicKey::from(&secret_key);

    //     // Convert the keys to string
    //     let secret_key_str =
    //         general_purpose::STANDARD.encode(format!("b{:?}", secret_key.to_bytes()));
    //     let public_key_str =
    //         general_purpose::STANDARD.encode(format!("b{:?}", public_key.to_bytes()));

    //     Ok(Self {
    //         version: 1,
    //         hid: hid.to_string(),
    //         xpub: public_key_str,
    //         xprv: secret_key_str,
    //         mnemonic: phrase.phrase().to_string(),
    //     })
    // }
}

// fn main() {
// Generate a random mnemonic
// let mnemonic = MnemonicBuilder::build_random(&self, &mut OsRng)
//     .entropy(Randomness::Strong)
//     .word_count(Words::Twelve)
//     .build()
//     .unwrap();

// Or, use an existing mnemonic
// let mnemonic = Mnemonic::from_phrase(
//     "YOUR_MNEMONIC_PHRASE_HERE"
// ).unwrap();

// println!("Mnemonic: {}", mnemonic.phrase());

// // Derive the wallet from the mnemonic
// let wallet = LocalWallet::from_mnemonic(mnemonic.clone(), None, 0, Network::Mainnet).unwrap();

// // Get the private key
// let private_key = wallet.private_key().to_pem().unwrap();
// println!("Private Key: {}", private_key.to_hex::<String>());

// // Get the public key
// let public_key = wallet.public_key().expect("Failed to get public key");
// println!("Public Key: {}", public_key.to_hex::<String>());

// // Recover the wallet from the mnemonic
// let recovered_wallet = LocalWallet::from_mnemonic(mnemonic, None, 0, Network::Mainnet).unwrap();
// assert_eq!(wallet.address(), recovered_wallet.address());
// println!("Successfully recovered wallet from mnemonic");
// }

#[cfg(test)]
mod tests {

    use bip32::{secp256k1::elliptic_curve::zeroize::__internal::AssertZeroize, Mnemonic};

    use crate::peer::Peer;

    #[test]
    fn create_peer() {
        let peer = Peer::new("lucid-mysterium".to_string(), &"password123".to_string());
    }

    #[test]
    fn test_new_eth() {
        let peer = Peer::new_eth();
    }

    #[test]
    fn get_words() {
        let mnemonic = Peer::get_words();
        println!("{}", mnemonic.phrase().to_string())
    }

    #[test]
    fn from_phrase() {
        let mut peer1 = Peer::from_phrase(
            "~lomder-librun".to_string(),
            Mnemonic::new("mention citizen dutch share final ship valid balance rack drastic mystery grief require fluid mom forget toast business snake laugh faint dentist ensure return", bip32::Language::English).unwrap(),
            &"password123".to_string()).unwrap();

        peer1.write_keyfile(peer1.root_xprv.clone());
        println!("hid: {}", peer1.hid);
        println!("xpub: {}", peer1.xpub);
        println!("xprv: {}", peer1.xprv);

        // zeroize the keys
        peer1.xprv.zeroize_or_on_drop();
        peer1.root_xprv.zeroize_or_on_drop();

        let mut peer2 = Peer::from_phrase(
            "~fasnut-famden".to_string(),
            Mnemonic::new("gas panel detail execute stairs crunch economy south truck lava mistake ladder source dry burger they barely off model abstract trim narrow they prosper", bip32::Language::English).unwrap(),
            &"password123".to_string()).unwrap();

        peer2.write_keyfile(peer2.root_xprv.clone());

        println!("hid: {}", peer2.hid);
        println!("xpub: {}", peer2.xpub);
        println!("xprv: {}", peer2.xprv);

        // zeroize the keys
        peer2.xprv.zeroize_or_on_drop();
        peer2.root_xprv.zeroize_or_on_drop();
    }
}
