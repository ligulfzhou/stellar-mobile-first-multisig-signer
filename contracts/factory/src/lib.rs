#![allow(deprecated)]
#![no_std]
 
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{
    contract, contractimpl, contracttype, contracterror,
    Address, BytesN, Env, Symbol, Vec, Bytes,
    token::Client as TokenClient,
};
 
#[contracttype]
#[derive(Clone)]
pub struct FactoryConfig {
    pub admin: Address,
    pub vault_wasm_hash: BytesN<32>,
    pub fee_token: Address,
    pub fee_amount: i128,
    pub fee_recipient: Address,
    pub total_vaults_created: u64,
}
 
#[contracttype]
#[derive(Clone)]
pub struct PendingUpgrade {
    pub new_hash: BytesN<32>,
    pub activate_at: u64,
}
 
#[contracttype]
pub enum StorageKey {
    Config,
    PendingUpgrade,
    Initialized,
}
 
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum FactoryError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    NotAdmin = 3,
    InvalidThreshold = 4,
    NoSigners = 5,
    DuplicateSigner = 6,
    CreatorNotInSigners = 7,
    UpgradeNotReady = 8,
    NoUpgradePending = 9,
    InvalidFeeAmount = 10,
}
 
const UPGRADE_DELAY_LEDGERS: u64 = 17_280;
const MAX_SIGNERS: u32 = 20;
 
mod vault {
    soroban_sdk::contractimport!(
        file = "../target/wasm32v1-none/release/multisig_vault.wasm"
    );
}
 
#[contract]
pub struct VaultFactory;
 
#[contractimpl]
impl VaultFactory {
    pub fn initialize(
        env: Env,
        admin: Address,
        vault_wasm_hash: BytesN<32>,
        fee_token: Address,
        fee_amount: i128,
        fee_recipient: Address,
    ) -> Result<(), FactoryError> {
        if env.storage().instance().has(&StorageKey::Initialized) {
            return Err(FactoryError::AlreadyInitialized);
        }
        admin.require_auth();
        if fee_amount < 0 {
            return Err(FactoryError::InvalidFeeAmount);
        }
        let config = FactoryConfig {
            admin: admin.clone(),
            vault_wasm_hash,
            fee_token,
            fee_amount,
            fee_recipient,
            total_vaults_created: 0,
        };
        env.storage().instance().set(&StorageKey::Config, &config);
        env.storage().instance().set(&StorageKey::Initialized, &true);
        env.events().publish(
            (Symbol::new(&env, "factory_initialized"),),
            (env.current_contract_address(), admin),
        );
        Ok(())
    }
 
    pub fn create_vault(
        env: Env,
        creator: Address,
        name: Symbol,
        signers: Vec<Address>,
        threshold: u32,
    ) -> Result<Address, FactoryError> {
        creator.require_auth();
 
        if signers.is_empty() {
            return Err(FactoryError::NoSigners);
        }
        let signer_count = signers.len() as u32;
        if signer_count > MAX_SIGNERS {
            return Err(FactoryError::NoSigners);
        }
        if threshold == 0 || threshold > signer_count {
            return Err(FactoryError::InvalidThreshold);
        }
 
        let mut creator_found = false;
        for s in signers.iter() {
            if s == creator {
                creator_found = true;
                break;
            }
        }
        if !creator_found {
            return Err(FactoryError::CreatorNotInSigners);
        }
 
        Self::validate_no_duplicates(&signers)?;
 
        let mut config: FactoryConfig = env
            .storage()
            .instance()
            .get(&StorageKey::Config)
            .ok_or(FactoryError::NotInitialized)?;
 
        // Deterministic salt: counter + creator ONLY
        // No timestamp, no prng, no ledger sequence
        // This guarantees identical output between simulation and execution
        let salt = Self::generate_salt(&env, config.total_vaults_created, &creator);
 
        let deployed_address = env
            .deployer()
            .with_current_contract(salt)
            .deploy(config.vault_wasm_hash.clone());
 
        // Initialize vault with fee config from factory
        let vault_client = vault::Client::new(&env, &deployed_address);
        vault_client.initialize(
            &name,
            &signers,
            &threshold,
            &config.fee_recipient,
            &config.fee_token,
        );
 
        // Collect fee after successful deployment
        if config.fee_amount > 0 {
            let token = TokenClient::new(&env, &config.fee_token);
            token.transfer(&creator, &config.fee_recipient, &config.fee_amount);
        }
 
        config.total_vaults_created += 1;
        env.storage().instance().set(&StorageKey::Config, &config);
 
        env.events().publish(
            (Symbol::new(&env, "vault_created"),),
            (deployed_address.clone(), creator, name, signers, threshold, env.ledger().timestamp()),
        );
 
        Ok(deployed_address)
    }
 
    pub fn propose_wasm_upgrade(
        env: Env, admin: Address, new_hash: BytesN<32>,
    ) -> Result<(), FactoryError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        let pending = PendingUpgrade {
            new_hash: new_hash.clone(),
            activate_at: env.ledger().timestamp() + UPGRADE_DELAY_LEDGERS,
        };
        env.storage().instance().set(&StorageKey::PendingUpgrade, &pending);
        env.events().publish((Symbol::new(&env, "wasm_upgrade_proposed"),), (new_hash, pending.activate_at));
        Ok(())
    }
 
    pub fn execute_wasm_upgrade(env: Env, admin: Address) -> Result<(), FactoryError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        let pending: PendingUpgrade = env.storage().instance().get(&StorageKey::PendingUpgrade)
            .ok_or(FactoryError::NoUpgradePending)?;
        if env.ledger().timestamp() < pending.activate_at {
            return Err(FactoryError::UpgradeNotReady);
        }
        let mut config: FactoryConfig = env.storage().instance().get(&StorageKey::Config).unwrap();
        config.vault_wasm_hash = pending.new_hash.clone();
        env.storage().instance().set(&StorageKey::Config, &config);
        env.storage().instance().remove(&StorageKey::PendingUpgrade);
        env.events().publish((Symbol::new(&env, "wasm_upgrade_executed"),), pending.new_hash);
        Ok(())
    }
 
    pub fn cancel_wasm_upgrade(env: Env, admin: Address) -> Result<(), FactoryError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        if !env.storage().instance().has(&StorageKey::PendingUpgrade) {
            return Err(FactoryError::NoUpgradePending);
        }
        env.storage().instance().remove(&StorageKey::PendingUpgrade);
        env.events().publish((Symbol::new(&env, "wasm_upgrade_cancelled"),), admin);
        Ok(())
    }
 
    pub fn set_fee(env: Env, admin: Address, new_fee_amount: i128) -> Result<(), FactoryError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        if new_fee_amount < 0 { return Err(FactoryError::InvalidFeeAmount); }
        let mut config: FactoryConfig = env.storage().instance().get(&StorageKey::Config).unwrap();
        config.fee_amount = new_fee_amount;
        env.storage().instance().set(&StorageKey::Config, &config);
        env.events().publish((Symbol::new(&env, "fee_updated"),), new_fee_amount);
        Ok(())
    }
 
    pub fn set_fee_token(env: Env, admin: Address, new_fee_token: Address) -> Result<(), FactoryError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        let mut config: FactoryConfig = env.storage().instance().get(&StorageKey::Config).unwrap();
        config.fee_token = new_fee_token.clone();
        env.storage().instance().set(&StorageKey::Config, &config);
        env.events().publish((Symbol::new(&env, "fee_token_updated"),), new_fee_token);
        Ok(())
    }
 
    pub fn set_fee_recipient(env: Env, admin: Address, new_recipient: Address) -> Result<(), FactoryError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        let mut config: FactoryConfig = env.storage().instance().get(&StorageKey::Config).unwrap();
        config.fee_recipient = new_recipient.clone();
        env.storage().instance().set(&StorageKey::Config, &config);
        env.events().publish((Symbol::new(&env, "fee_recipient_updated"),), new_recipient);
        Ok(())
    }
 
    pub fn set_admin(env: Env, admin: Address, new_admin: Address) -> Result<(), FactoryError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        let mut config: FactoryConfig = env.storage().instance().get(&StorageKey::Config).unwrap();
        config.admin = new_admin.clone();
        env.storage().instance().set(&StorageKey::Config, &config);
        env.events().publish((Symbol::new(&env, "admin_transferred"),), (admin, new_admin));
        Ok(())
    }
 
    pub fn set_vault_wasm_hash(env: Env, admin: Address, new_hash: BytesN<32>) -> Result<(), FactoryError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        let mut config: FactoryConfig = env.storage().instance().get(&StorageKey::Config).unwrap();
        config.vault_wasm_hash = new_hash.clone();
        env.storage().instance().set(&StorageKey::Config, &config);
        env.events().publish((Symbol::new(&env, "wasm_hash_updated"),), new_hash);
        Ok(())
    }
 
    pub fn get_config(env: Env) -> Option<FactoryConfig> {
        env.storage().instance().get(&StorageKey::Config)
    }
 
    pub fn get_vault_count(env: Env) -> u64 {
        env.storage().instance().get(&StorageKey::Config)
            .map(|c: FactoryConfig| c.total_vaults_created).unwrap_or(0)
    }
 
    pub fn get_fee(env: Env) -> i128 {
        env.storage().instance().get(&StorageKey::Config)
            .map(|c: FactoryConfig| c.fee_amount).unwrap_or(0)
    }
 
    pub fn get_pending_upgrade(env: Env) -> Option<PendingUpgrade> {
        env.storage().instance().get(&StorageKey::PendingUpgrade)
    }
 
    fn require_admin(env: &Env, address: &Address) -> Result<(), FactoryError> {
        let config: FactoryConfig = env.storage().instance().get(&StorageKey::Config)
            .ok_or(FactoryError::NotInitialized)?;
        if config.admin != *address { return Err(FactoryError::NotAdmin); }
        Ok(())
    }
 
    // Deterministic salt: only counter + creator address
    // No timestamp, no ledger sequence, no PRNG
    fn generate_salt(env: &Env, count: u64, creator: &Address) -> BytesN<32> {
        let mut salt_bytes = Bytes::new(env);
        let count_bytes = count.to_be_bytes();
        for byte in count_bytes.iter() {
            salt_bytes.push_back(*byte);
        }
        let creator_bytes: Bytes = creator.clone().to_xdr(env);
        salt_bytes.append(&creator_bytes);
        let hash = env.crypto().sha256(&salt_bytes);
        BytesN::from_array(env, &hash.to_array())
    }
 
    fn validate_no_duplicates(signers: &Vec<Address>) -> Result<(), FactoryError> {
        let len = signers.len();
        for i in 0..len {
            for j in (i + 1)..len {
                if signers.get(i) == signers.get(j) { return Err(FactoryError::DuplicateSigner); }
            }
        }
        Ok(())
    }
}