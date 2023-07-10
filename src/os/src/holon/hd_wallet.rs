use hdwallet::{DefaultKeyChain, ExtendedPrivKey, KeyChain};
use secp256k1::{ecdsa::Signature, Message, PublicKey, Secp256k1, SecretKey};
use serde::{Deserialize, Serialize};
use sha2::digest::generic_array::typenum::U32;
use sha2::digest::generic_array::GenericArray;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KeyPair {
    pub derivation: String,
    pub pubkey: [u8; 32],
    privkey: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Keys {
    eth: KeyPair,
    networking: KeyPair,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HDWallet {
    master_key: [u8; 32],
    keys: Keys,
}

fn derive_eth_keypair(key_chain: &DefaultKeyChain) -> KeyPair {
    let derivation_path = "m/44'/60'/0'/0/0";
    let (derived_key, _) = key_chain
        .derive_private_key(derivation_path.into())
        .unwrap();
    let context = Secp256k1::new();
    let private_key = derived_key.private_key;
    let public_key = PublicKey::from_secret_key(&context, &private_key);
    KeyPair {
        derivation: derivation_path.into(),
        pubkey: public_key.serialize()[1..].try_into().unwrap(),
        privkey: private_key.secret_bytes(),
    }
}

fn derive_networking_keypair(key_chain: &DefaultKeyChain) -> KeyPair {
    let derivation_path = "m/44'/200'/0'/0/0";
    let (derived_key, _) = key_chain
        .derive_private_key(derivation_path.into())
        .unwrap();
    let context = Secp256k1::new();
    let private_key = derived_key.private_key;
    let public_key = PublicKey::from_secret_key(&context, &private_key);

    KeyPair {
        derivation: derivation_path.into(),
        pubkey: public_key.serialize()[1..].try_into().unwrap(),
        privkey: private_key.secret_bytes(),
    }
}

impl HDWallet {
    /// # Examples
    ///
    /// ```rust
    /// use conduit::holon::hd_wallet::*;
    /// use bip32::Mnemonic;
    /// use rand;
    ///
    /// let mut rng = rand::thread_rng();
    ///
    /// let password: &str = "password123";
    /// let mnemonic_phrase = Mnemonic::random(rng, bip32::Language::English);
    /// let seed = mnemonic_phrase.to_seed(password);
    ///
    /// let hd_wallet = HDWallet::new(seed.as_bytes());
    /// ```
    pub fn new(seed: &[u8]) -> Self {
        let master_key = ExtendedPrivKey::with_seed(seed).expect("Failed to create root key");
        let master_key_bytes = master_key.private_key.secret_bytes();
        // println!("seed: {:?}", hex::encode(seed));

        let key_chain = DefaultKeyChain::new(master_key);

        let eth_keypair = derive_eth_keypair(&key_chain);
        let networking_keypair = derive_networking_keypair(&key_chain);

        HDWallet {
            master_key: master_key_bytes,
            keys: Keys {
                eth: eth_keypair,
                networking: networking_keypair,
            },
        }
    }

    /// # Examples
    /// ```rust
    /// use conduit::holon::hd_wallet::*;
    /// use bip32::Mnemonic;
    /// use rand;
    ///
    /// let mut rng = rand::thread_rng();
    /// let password: &str = "password123";
    /// let mnemonic_phrase = Mnemonic::random(rng, bip32::Language::English);
    /// let seed = mnemonic_phrase.to_seed(password);
    /// let hd_wallet = HDWallet::new(seed.as_bytes());
    ///
    /// hd_wallet.sign_message("hello world".as_bytes());
    ///
    pub fn sign_message(&self, message: &[u8]) -> [u8; 64] {
        let hashed_message = HDWallet::hash_message(message);
        let context = Secp256k1::new();
        let secret_key = SecretKey::from_slice(&self.keys.eth.privkey).unwrap();
        println!("{:?}", secret_key.display_secret());
        let message = Message::from_slice(hashed_message.as_slice())
            .expect("message creation failed from_slice");
        println!("message: {:?}", message.to_string());

        let signature = context.sign_ecdsa(&message, &secret_key);
        signature.serialize_compact()
    }

    fn hash_message(message: &[u8]) -> [u8; 32] {
        let mut hasher: Sha256 = Digest::new();
        // write input message
        hasher.update(message);
        // read hash digest and consume hasher
        let result: GenericArray<u8, U32> = hasher.finalize();
        let arr: [u8; 32] = result.into();
        arr
    }

    pub fn verify_message(&self, message: &[u8], signature: &[u8; 64]) -> bool {
        let hashed_message = HDWallet::hash_message(message);
        let context = Secp256k1::new();
        let public_key = PublicKey::from_slice(&self.keys.eth.pubkey).unwrap();
        let message = Message::from_slice(hashed_message.as_slice())
            .expect("message creation failed from_slice");
        let signature = Signature::from_compact(signature).unwrap();
        context
            .verify_ecdsa(&message, &signature, &public_key)
            .is_ok()
    }

    pub fn get_eth_pubkey(&self) -> [u8; 32] {
        self.keys.eth.pubkey
    }

    /// # Examples
    ///
    /// ```rust
    /// use holon::hd_wallet::HDWallet;
    /// use bip32::Mnemonic;
    ///
    ///
    /// let mnemonic_phrase = Mnemonic::new("mention citizen dutch share final ship valid balance rack drastic mystery grief", bip32::Language::English).unwrap()
    ///
    /// ```
    pub fn get_networking_pubkey(&self) -> [u8; 32] {
        self.keys.networking.pubkey
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bip32::Seed;
    use rand::rngs::OsRng;
    use rand_core::RngCore;
    use secp256k1::SecretKey;

    #[test]
    fn it_creates_hd_wallet() {
        // generate random seed
        let mut seed = [0u8; 32];
        OsRng.fill_bytes(&mut seed);

        // generate test wallet
        let test_wallet = HDWallet::new(&seed);

        // Generate test master key
        let test_master_key = ExtendedPrivKey::with_seed(&seed).expect("Failed to create root key");

        assert_eq!(
            test_wallet.master_key,
            test_master_key.private_key.secret_bytes()
        );
    }

    #[test]
    fn it_derives_eth_keypair() {
        // generate random seed
        let mut seed = [0u8; 32];
        OsRng.fill_bytes(&mut seed);

        let master_key = ExtendedPrivKey::with_seed(&seed).expect("Failed to create root key");
        let key_chain = DefaultKeyChain::new(master_key.clone());

        let eth_keypair = derive_eth_keypair(&key_chain);

        let context = Secp256k1::new();
        let public_key = PublicKey::from_secret_key(
            &context,
            &SecretKey::from_slice(&eth_keypair.privkey).unwrap(),
        );

        let test_pubkey: [u8; 32] = public_key.serialize()[1..].try_into().unwrap();

        assert_eq!(eth_keypair.pubkey, test_pubkey);
        assert_eq!(eth_keypair.derivation, "m/44'/60'/0'/0/0");
    }

    #[test]
    fn it_derives_networking_keypair() {
        // generate random seed
        let mut seed = [0u8; 32];
        OsRng.fill_bytes(&mut seed);

        let master_key = ExtendedPrivKey::with_seed(&seed).expect("Failed to create root key");
        let key_chain = DefaultKeyChain::new(master_key.clone());

        let networking_keypair = derive_networking_keypair(&key_chain);

        let context = Secp256k1::new();
        let public_key = PublicKey::from_secret_key(
            &context,
            &SecretKey::from_slice(&networking_keypair.privkey).unwrap(),
        );
        let test_pubkey: [u8; 32] = public_key.serialize()[1..].try_into().unwrap();

        assert_eq!(networking_keypair.pubkey, test_pubkey);
        assert_eq!(networking_keypair.derivation, "m/44'/200'/0'/0/0");
    }

    #[test]
    fn it_signs_messages() {
        // generate random seed
        let mut seed = [0u8; 32];
        OsRng.fill_bytes(&mut seed);

        let master_key = ExtendedPrivKey::with_seed(&seed).expect("Failed to create root key");
        let key_chain = DefaultKeyChain::new(master_key.clone());

        let hd_wallet = HDWallet::new(&seed);

        let message = "hello world".as_bytes();

        let signature = hd_wallet.sign_message(message);
        // manually hash and sign message
        let mut hasher: Sha256 = Digest::new();
        hasher.update(message);
        let result: GenericArray<u8, U32> = hasher.finalize();
        let arr: [u8; 32] = result.into();

        let context = Secp256k1::new();
        let secret_key = secp256k1::SecretKey::from_slice(&hd_wallet.keys.eth.privkey).unwrap();
        let message = secp256k1::Message::from_slice(arr.as_slice()).unwrap();
        let test_signature = context.sign_ecdsa(&message, &secret_key);

        assert_eq!(signature, test_signature.serialize_compact());
    }
}
