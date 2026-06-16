//! Factory contract client for deploying new vault instances.

use {
    crate::{config::DEFAULT_WRITE_FEE, tx::build_invoke_tx, VaultClient},
    anyhow::{anyhow, Result},
    stellar_core::{
        address_to_scval, scval_to_address_string, sign_and_submit, symbol_to_scval, u32_to_scval,
        Keypair, Network, RpcClient, poll_transaction_return,
    },
    soroban_client::xdr::{ScVal, ScVec},
};

pub const MAX_VAULT_SIGNERS: u32 = 20;

/// Demo factory deployed on testnet (see `deploy/testnet.json`).
pub const FACTORY_TESTNET: &str = "CALJDBAQ3MGJUPV23K5CW3MQU5PREERN7POQ7RQMSJ7XLJU5U5J6AF6L";

pub struct FactoryClient {
    pub factory: String,
    pub rpc: RpcClient,
    pub network: Network,
    pub rpc_url: String,
    pub horizon_url: String,
}

impl FactoryClient {
    pub fn new(factory: impl Into<String>, rpc_url: impl Into<String>, network: Network) -> Result<Self> {
        let factory = factory.into();
        let rpc_url = rpc_url.into();
        let rpc = RpcClient::new(&rpc_url, network)?;
        let horizon_url = network.default_horizon_url().to_string();
        Ok(Self {
            factory,
            rpc,
            network,
            rpc_url,
            horizon_url,
        })
    }

    pub fn testnet(factory: impl Into<String>) -> Result<Self> {
        Self::new(factory, Network::Testnet.default_rpc_url(), Network::Testnet)
    }

    /// Create a new N-of-M vault. Creator must appear in `signers`.
    pub async fn create_vault(
        &self,
        keypair: &Keypair,
        name: &str,
        signers: &[String],
        threshold: u32,
    ) -> Result<String> {
        validate_create_params(&keypair.public_key(), signers, threshold)?;

        let creator = keypair.public_key();
        let unsigned = self
            .build_create_vault_tx(&creator, name, signers, threshold)
            .await?;
        let hash = sign_and_submit(&self.rpc_url, self.network, keypair, &unsigned).await?;
        let return_val = poll_transaction_return(&self.rpc_url, &hash).await?;
        scval_to_address_string(&return_val)
    }

    pub async fn build_create_vault_tx(
        &self,
        creator: &str,
        name: &str,
        signers: &[String],
        threshold: u32,
    ) -> Result<String> {
        validate_create_params(creator, signers, threshold)?;
        build_invoke_tx(
            &self.rpc,
            self.network,
            &self.horizon_url,
            &self.factory,
            creator,
            "create_vault",
            vec![
                address_to_scval(creator)?,
                symbol_to_scval(name)?,
                address_vec_to_scval(signers)?,
                u32_to_scval(threshold),
            ],
            DEFAULT_WRITE_FEE,
        )
        .await
    }
}

fn validate_create_params(creator: &str, signers: &[String], threshold: u32) -> Result<()> {
    if signers.is_empty() {
        return Err(anyhow!("at least one signer required"));
    }
    if signers.len() > MAX_VAULT_SIGNERS as usize {
        return Err(anyhow!("max {} signers", MAX_VAULT_SIGNERS));
    }
    if threshold == 0 || threshold > signers.len() as u32 {
        return Err(anyhow!("threshold must be 1..={}", signers.len()));
    }
    if !signers.iter().any(|s| s == creator) {
        return Err(anyhow!("creator must be included in signers"));
    }
    let mut unique = signers.to_vec();
    unique.sort();
    unique.dedup();
    if unique.len() != signers.len() {
        return Err(anyhow!("duplicate signers"));
    }
    Ok(())
}

fn address_vec_to_scval(addresses: &[String]) -> Result<ScVal> {
    let items: Result<Vec<ScVal>> = addresses.iter().map(|a| address_to_scval(a)).collect();
    let vec = ScVec::try_from(items?).map_err(|e| anyhow!("signer vec: {:?}", e))?;
    Ok(ScVal::Vec(Some(vec)))
}

/// Convenience: create vault then wrap as [`VaultClient`].
pub async fn create_vault_client(
    factory: &str,
    rpc_url: &str,
    network: Network,
    keypair: &Keypair,
    name: &str,
    signers: &[String],
    threshold: u32,
) -> Result<VaultClient> {
    let factory_client = FactoryClient::new(factory, rpc_url, network)?;
    let vault_id = factory_client
        .create_vault(keypair, name, signers, threshold)
        .await?;
    VaultClient::new(vault_id, rpc_url, network)
}
