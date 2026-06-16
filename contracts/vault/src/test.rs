#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, Symbol};

#[test]
fn test_initialize_vault() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MultisigVault);
    let client = MultisigVaultClient::new(&env, &contract_id);

    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let signer3 = Address::generate(&env);

    let signers = soroban_sdk::vec![&env, signer1, signer2, signer3];

    client.initialize(&Symbol::new(&env, "TestVault"), &signers, &2);

    let config = client.get_config();
    assert_eq!(config.threshold, 2);
    assert_eq!(config.signer_count, 3);
}

#[test]
fn test_propose_and_approve() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MultisigVault);
    let client = MultisigVaultClient::new(&env, &contract_id);

    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let signers = soroban_sdk::vec![&env, signer1.clone(), signer2.clone()];

    client.initialize(&Symbol::new(&env, "TestVault"), &signers, &2);

    let token = Address::generate(&env);
    let recipient = Address::generate(&env);

    // Propose
    let proposal_id = client.propose(&signer1, &token, &recipient, &1000);
    assert_eq!(proposal_id, 0);

    // Check proposal status
    let proposal = client.get_proposal(&proposal_id);
    assert_eq!(proposal.status, ProposalStatus::Pending);
    assert_eq!(proposal.approvals.len(), 1);

    // Second approval
    client.approve(&signer2, &proposal_id);

    // Check now approved
    let proposal = client.get_proposal(&proposal_id);
    assert_eq!(proposal.status, ProposalStatus::Approved);
}