use {
    anyhow::{anyhow, Result},
    bip39::{Language, Mnemonic},
    soroban_client::keypair::{Keypair as SorobanKeypair, KeypairBehavior},
};

/// Stellar Ed25519 keypair wrapper (adapted from stellar-arb/src/account.rs).
pub struct Keypair(SorobanKeypair);

impl Keypair {
    pub fn inner(&self) -> &SorobanKeypair {
        &self.0
    }

    pub fn public_key(&self) -> String {
        self.0.public_key()
    }

    pub fn from_secret(secret: &str) -> Result<Self> {
        let kp = SorobanKeypair::from_secret(secret).map_err(|e| anyhow!("invalid secret key: {:?}", e))?;
        Ok(Self(kp))
    }

    /// SEP-0005 derivation: m/44'/148'/index'
    pub fn from_mnemonic(phrase: &str, index: u32) -> Result<Self> {
        let mnemonic =
            Mnemonic::parse_in(Language::English, phrase).map_err(|e| anyhow!("invalid mnemonic: {:?}", e))?;
        let seed = mnemonic.to_seed("");
        let path = [0x80000000 + 44, 0x80000000 + 148, 0x80000000 + index];
        let derived = slip10_ed25519::derive_ed25519_private_key(&seed, &path);
        let kp =
            SorobanKeypair::from_raw_ed25519_seed(&derived).map_err(|e| anyhow!("key derivation failed: {:?}", e))?;
        Ok(Self(kp))
    }
}
