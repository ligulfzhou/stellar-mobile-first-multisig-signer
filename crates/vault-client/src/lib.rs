pub mod config;
pub mod factory;
pub mod read;
pub mod tx;
pub mod types;
pub mod write;

use {
    anyhow::Result,
    stellar_core::{network::Network, RpcClient},
};
pub use {config::*, factory::FactoryClient, read::VaultReader, types::*, write::VaultWriter};

/// High-level client for a single vault contract instance.
pub struct VaultClient {
    pub vault: String,
    pub rpc: RpcClient,
    pub network: Network,
    pub rpc_url: String,
    pub horizon_url: String,
}

impl VaultClient {
    pub fn new(vault: impl Into<String>, rpc_url: impl Into<String>, network: Network) -> Result<Self> {
        let vault = vault.into();
        let rpc_url = rpc_url.into();
        let rpc = RpcClient::new(&rpc_url, network)?;
        let horizon_url = network.default_horizon_url().to_string();
        Ok(Self {
            vault,
            rpc,
            network,
            rpc_url,
            horizon_url,
        })
    }

    pub fn testnet(vault: impl Into<String>) -> Result<Self> {
        Self::new(vault, Network::Testnet.default_rpc_url(), Network::Testnet)
    }

    pub fn reader(&self) -> VaultReader<'_> {
        VaultReader { client: self }
    }

    pub fn writer(&self) -> VaultWriter<'_> {
        VaultWriter { client: self }
    }
}
