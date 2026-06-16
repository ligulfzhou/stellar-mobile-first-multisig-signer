//! Stellar Vault testnet constants (from
//! Stellar_Vault-fork/dashboard/src/config.ts).

pub const FACTORY_CONTRACT_ID: &str = "CCNGOW6UCZKELBAR377HDHWAJJLKD6SJHUFCDT4UM6M2AYPSOEBYLDVA";
pub const REGISTRY_CONTRACT_ID: &str = "CDJCQNXYTWZ3VF2FL2MCWMZB6RPQYRAFNNO6KEKW2MN7ALXGB5SGYTJ4";
pub const NATIVE_TOKEN: &str = "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC";

/// Public test vault from Stellar Vault docs (testnet).
pub const TEST_VAULT: &str = "CBJ4BFOUDMQWFPCBALQTO2565STNGFMGQWDYVQ7MBWRZF5WSI2Z4VT5W";

/// Default Soroban resource fee for vault write operations (matches dashboard).
pub const DEFAULT_WRITE_FEE: u32 = 10_000_000;
