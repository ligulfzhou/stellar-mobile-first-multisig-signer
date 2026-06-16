#![no_std]
#![allow(deprecated)]

use soroban_sdk::{
    contract, contractimpl, contracttype, contracterror,
    Address, BytesN, Env, Symbol, Vec,
};

// =============================================================================
// Data Structures
// =============================================================================

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum FactoryCapability {
    Basic = 0,           // Basic vault operations
    BatchOps = 1,        // Batch lock creation
    SpendLimits = 2,     // Daily spend limits
    SuperAdmin = 3,      // SuperAdmin role support
    TimeLocks = 4,       // Time-locked assets
    Vesting = 5,         // Vesting schedules
    PublicView = 6,      // Public vault view
    Proposals = 7,       // Threshold proposals
}

#[contracttype]
#[derive(Clone)]
pub struct FactoryInfo {
    pub address: Address,
    pub version: u32,
    pub wasm_hash: BytesN<32>,
    pub capabilities: Vec<FactoryCapability>,
    pub created_at: u64,
    pub is_active: bool,
    pub total_vaults: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct RegistryConfig {
    pub admin: Address,
    pub total_factories: u32,
    pub default_factory_version: u32,
}

#[contracttype]
pub enum StorageKey {
    Config,
    Initialized,
    Factory(u32),           // version -> FactoryInfo
    FactoryByAddress(Address), // address -> version
    AllVersions,            // Vec<u32> of all versions
}

// =============================================================================
// Errors
// =============================================================================

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RegistryError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    NotAuthorized = 3,
    FactoryNotFound = 4,
    FactoryAlreadyExists = 5,
    InvalidVersion = 6,
    FactoryNotActive = 7,
}

// =============================================================================
// Contract
// =============================================================================

#[contract]
pub struct VaultRegistry;

#[contractimpl]
impl VaultRegistry {
    // ============ INITIALIZATION ============
    pub fn initialize(env: Env, admin: Address) -> Result<(), RegistryError> {
        if env.storage().instance().has(&StorageKey::Initialized) {
            return Err(RegistryError::AlreadyInitialized);
        }

        let config = RegistryConfig {
            admin: admin.clone(),
            total_factories: 0,
            default_factory_version: 0,
        };

        env.storage().instance().set(&StorageKey::Config, &config);
        env.storage().instance().set(&StorageKey::AllVersions, &Vec::<u32>::new(&env));
        env.storage().instance().set(&StorageKey::Initialized, &true);

        env.events().publish(
            (Symbol::new(&env, "registry_init"),),
            admin,
        );

        Ok(())
    }

    // ============ FACTORY MANAGEMENT ============
    
    /// Register a new factory version
    pub fn register_factory(
        env: Env,
        admin: Address,
        factory_address: Address,
        version: u32,
        wasm_hash: BytesN<32>,
        capabilities: Vec<FactoryCapability>,
    ) -> Result<(), RegistryError> {
        admin.require_auth();
        Self::require_initialized(&env)?;
        Self::require_admin(&env, &admin)?;

        // Check factory doesn't already exist
        if env.storage().persistent().has(&StorageKey::Factory(version)) {
            return Err(RegistryError::FactoryAlreadyExists);
        }

        let factory_info = FactoryInfo {
            address: factory_address.clone(),
            version,
            wasm_hash,
            capabilities,
            created_at: env.ledger().timestamp(),
            is_active: true,
            total_vaults: 0,
        };

        // Store factory info
        env.storage().persistent().set(&StorageKey::Factory(version), &factory_info);
        env.storage().persistent().set(&StorageKey::FactoryByAddress(factory_address.clone()), &version);

        // Update versions list
        let mut versions: Vec<u32> = env.storage().instance()
            .get(&StorageKey::AllVersions)
            .unwrap_or(Vec::new(&env));
        versions.push_back(version);
        env.storage().instance().set(&StorageKey::AllVersions, &versions);

        // Update config
        let mut config: RegistryConfig = env.storage().instance().get(&StorageKey::Config).unwrap();
        config.total_factories += 1;
        if config.default_factory_version == 0 || version > config.default_factory_version {
            config.default_factory_version = version;
        }
        env.storage().instance().set(&StorageKey::Config, &config);

        env.events().publish(
            (Symbol::new(&env, "factory_registered"),),
            (factory_address, version),
        );

        Ok(())
    }

    /// Deactivate a factory (no new vaults can be created)
    pub fn deactivate_factory(
        env: Env,
        admin: Address,
        version: u32,
    ) -> Result<(), RegistryError> {
        admin.require_auth();
        Self::require_initialized(&env)?;
        Self::require_admin(&env, &admin)?;

        let mut factory: FactoryInfo = env.storage().persistent()
            .get(&StorageKey::Factory(version))
            .ok_or(RegistryError::FactoryNotFound)?;

        factory.is_active = false;
        env.storage().persistent().set(&StorageKey::Factory(version), &factory);

        env.events().publish(
            (Symbol::new(&env, "factory_deactivated"),),
            version,
        );

        Ok(())
    }

    /// Reactivate a factory
    pub fn activate_factory(
        env: Env,
        admin: Address,
        version: u32,
    ) -> Result<(), RegistryError> {
        admin.require_auth();
        Self::require_initialized(&env)?;
        Self::require_admin(&env, &admin)?;

        let mut factory: FactoryInfo = env.storage().persistent()
            .get(&StorageKey::Factory(version))
            .ok_or(RegistryError::FactoryNotFound)?;

        factory.is_active = true;
        env.storage().persistent().set(&StorageKey::Factory(version), &factory);

        env.events().publish(
            (Symbol::new(&env, "factory_activated"),),
            version,
        );

        Ok(())
    }

    /// Set the default factory version for new vaults
    pub fn set_default_version(
        env: Env,
        admin: Address,
        version: u32,
    ) -> Result<(), RegistryError> {
        admin.require_auth();
        Self::require_initialized(&env)?;
        Self::require_admin(&env, &admin)?;

        // Verify factory exists and is active
        let factory: FactoryInfo = env.storage().persistent()
            .get(&StorageKey::Factory(version))
            .ok_or(RegistryError::FactoryNotFound)?;

        if !factory.is_active {
            return Err(RegistryError::FactoryNotActive);
        }

        let mut config: RegistryConfig = env.storage().instance().get(&StorageKey::Config).unwrap();
        config.default_factory_version = version;
        env.storage().instance().set(&StorageKey::Config, &config);

        env.events().publish(
            (Symbol::new(&env, "default_version_set"),),
            version,
        );

        Ok(())
    }

    /// Update vault count for a factory (called by factory after vault creation)
    pub fn increment_vault_count(
        env: Env,
        factory_address: Address,
    ) -> Result<(), RegistryError> {
        factory_address.require_auth();
        Self::require_initialized(&env)?;

        let version: u32 = env.storage().persistent()
            .get(&StorageKey::FactoryByAddress(factory_address))
            .ok_or(RegistryError::FactoryNotFound)?;

        let mut factory: FactoryInfo = env.storage().persistent()
            .get(&StorageKey::Factory(version))
            .ok_or(RegistryError::FactoryNotFound)?;

        factory.total_vaults += 1;
        env.storage().persistent().set(&StorageKey::Factory(version), &factory);

        Ok(())
    }

    // ============ ADMIN MANAGEMENT ============
    
    pub fn transfer_admin(
        env: Env,
        admin: Address,
        new_admin: Address,
    ) -> Result<(), RegistryError> {
        admin.require_auth();
        Self::require_initialized(&env)?;
        Self::require_admin(&env, &admin)?;

        let mut config: RegistryConfig = env.storage().instance().get(&StorageKey::Config).unwrap();
        config.admin = new_admin.clone();
        env.storage().instance().set(&StorageKey::Config, &config);

        env.events().publish(
            (Symbol::new(&env, "admin_transferred"),),
            (admin, new_admin),
        );

        Ok(())
    }

    // ============ VIEW FUNCTIONS ============
    
    pub fn get_config(env: Env) -> Result<RegistryConfig, RegistryError> {
        Self::require_initialized(&env)?;
        Ok(env.storage().instance().get(&StorageKey::Config).unwrap())
    }

    pub fn get_factory(env: Env, version: u32) -> Result<FactoryInfo, RegistryError> {
        Self::require_initialized(&env)?;
        env.storage().persistent()
            .get(&StorageKey::Factory(version))
            .ok_or(RegistryError::FactoryNotFound)
    }

    pub fn get_factory_by_address(env: Env, address: Address) -> Result<FactoryInfo, RegistryError> {
        Self::require_initialized(&env)?;
        let version: u32 = env.storage().persistent()
            .get(&StorageKey::FactoryByAddress(address))
            .ok_or(RegistryError::FactoryNotFound)?;
        Self::get_factory(env, version)
    }

    pub fn get_default_factory(env: Env) -> Result<FactoryInfo, RegistryError> {
        Self::require_initialized(&env)?;
        let config: RegistryConfig = env.storage().instance().get(&StorageKey::Config).unwrap();
        if config.default_factory_version == 0 {
            return Err(RegistryError::FactoryNotFound);
        }
        Self::get_factory(env, config.default_factory_version)
    }

    pub fn get_all_versions(env: Env) -> Result<Vec<u32>, RegistryError> {
        Self::require_initialized(&env)?;
        Ok(env.storage().instance()
            .get(&StorageKey::AllVersions)
            .unwrap_or(Vec::new(&env)))
    }

    pub fn get_active_factories(env: Env) -> Result<Vec<FactoryInfo>, RegistryError> {
        Self::require_initialized(&env)?;
        let versions: Vec<u32> = env.storage().instance()
            .get(&StorageKey::AllVersions)
            .unwrap_or(Vec::new(&env));

        let mut active = Vec::new(&env);
        for version in versions.iter() {
            if let Some(factory) = env.storage().persistent().get::<StorageKey, FactoryInfo>(&StorageKey::Factory(version)) {
                if factory.is_active {
                    active.push_back(factory);
                }
            }
        }
        Ok(active)
    }

    /// Check if a factory has a specific capability
    pub fn has_capability(
        env: Env,
        version: u32,
        capability: FactoryCapability,
    ) -> Result<bool, RegistryError> {
        Self::require_initialized(&env)?;
        let factory: FactoryInfo = env.storage().persistent()
            .get(&StorageKey::Factory(version))
            .ok_or(RegistryError::FactoryNotFound)?;

        Ok(factory.capabilities.contains(&capability))
    }

    // ============ HELPERS ============
    
    fn require_initialized(env: &Env) -> Result<(), RegistryError> {
        if !env.storage().instance().has(&StorageKey::Initialized) {
            return Err(RegistryError::NotInitialized);
        }
        Ok(())
    }

    fn require_admin(env: &Env, address: &Address) -> Result<(), RegistryError> {
        let config: RegistryConfig = env.storage().instance().get(&StorageKey::Config).unwrap();
        if config.admin != *address {
            return Err(RegistryError::NotAuthorized);
        }
        Ok(())
    }
}