#![cfg(test)]

use crate::{VaultFactory, VaultFactoryClient};
use soroban_sdk::{
    testutils::Address as _,
    Address, Env, Symbol, Vec,
};

// Import the vault contract for testing
mod vault_contract {
    soroban_sdk::contractimport!(
        file = "../target/wasm32v1-none/release/multisig_vault.wasm"
    );
}

#[test]
fn test_factory_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let fee_token = Address::generate(&env);
    
    let factory_id = env.register(VaultFactory, ());
    let factory_client = VaultFactoryClient::new(&env, &factory_id);

    // Upload vault wasm
    let vault_wasm_hash = env.deployer().upload_contract_wasm(vault_contract::WASM);

    // Initialize factory
    factory_client.initialize(
        &admin,
        &vault_wasm_hash,
        &fee_token,
        &0i128,
        &admin,
    );

    let config = factory_client.get_config();
    assert_eq!(config.admin, admin);
    assert_eq!(config.fee_amount, 0);
    assert_eq!(config.total_vaults_created, 0);
}

#[test]
fn test_create_vault() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let fee_token = Address::generate(&env);
    let creator = Address::generate(&env);
    let signer1 = Address::generate(&env);

    let factory_id = env.register(VaultFactory, ());
    let factory_client = VaultFactoryClient::new(&env, &factory_id);

    // Upload vault wasm
    let vault_wasm_hash = env.deployer().upload_contract_wasm(vault_contract::WASM);

    // Initialize factory
    factory_client.initialize(
        &admin,
        &vault_wasm_hash,
        &fee_token,
        &0i128,
        &admin,
    );

    // Create signers vec
    let signers = Vec::from_array(&env, [signer1.clone()]);

    // Create vault
    let vault_address = factory_client.create_vault(
        &creator,
        &Symbol::new(&env, "TESTVAULT"),
        &signers,
        &1u32,
    );

    // Verify vault was created
    assert_eq!(factory_client.get_vault_count(), 1);
    
    let owner_vaults = factory_client.get_vaults_by_owner(&creator);
    assert_eq!(owner_vaults.len(), 1);
    assert_eq!(owner_vaults.get(0).unwrap(), vault_address);

    // Verify vault info
    let vault_info = factory_client.get_vault_info(&vault_address);
    assert!(vault_info.is_some());
    let info = vault_info.unwrap();
    assert_eq!(info.owner, creator);

    // Verify the deployed vault works
    let vault_client = vault_contract::Client::new(&env, &vault_address);
    let config = vault_client.get_config();
    assert_eq!(config.threshold, 1);
    assert_eq!(config.signer_count, 1);
}

#[test]
fn test_create_multiple_vaults() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let fee_token = Address::generate(&env);
    let creator = Address::generate(&env);
    let signer1 = Address::generate(&env);

    let factory_id = env.register(VaultFactory, ());
    let factory_client = VaultFactoryClient::new(&env, &factory_id);

    let vault_wasm_hash = env.deployer().upload_contract_wasm(vault_contract::WASM);

    factory_client.initialize(
        &admin,
        &vault_wasm_hash,
        &fee_token,
        &0i128,
        &admin,
    );

    let signers = Vec::from_array(&env, [signer1.clone()]);

    // Create first vault
    let vault1 = factory_client.create_vault(
        &creator,
        &Symbol::new(&env, "VAULT1"),
        &signers,
        &1u32,
    );

    // Create second vault
    let vault2 = factory_client.create_vault(
        &creator,
        &Symbol::new(&env, "VAULT2"),
        &signers,
        &1u32,
    );

    assert_eq!(factory_client.get_vault_count(), 2);
    assert!(vault1 != vault2);

    let owner_vaults = factory_client.get_vaults_by_owner(&creator);
    assert_eq!(owner_vaults.len(), 2);
}
