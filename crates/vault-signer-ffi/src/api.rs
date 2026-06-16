use {
    crate::{error::SignerError, runtime::block_on, types::{FfiProposal, FfiProposalSummary, FfiVaultConfig}},
    stellar_core::{network::Network, poll_transaction, sign_transaction_xdr, submit_signed_xdr, Keypair},
    vault_client::VaultClient,
};

fn rpc_url_for(network: &str, rpc_url: Option<String>) -> Result<String, SignerError> {
    if let Some(url) = rpc_url {
        return Ok(url);
    }
    let net = Network::parse(network)?;
    Ok(net.default_rpc_url().to_string())
}

fn client(vault: &str, network: &str, rpc_url: Option<String>) -> Result<VaultClient, SignerError> {
    let net = Network::parse(network)?;
    let url = rpc_url_for(network, rpc_url)?;
    VaultClient::new(vault, url, net).map_err(SignerError::from)
}

/// Mobile-facing API for Soroban multisig vault signing.
#[derive(uniffi::Object)]
pub struct VaultSigner;

#[uniffi::export]
impl VaultSigner {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self
    }

    /// Derive a Stellar public key (G...) from a BIP39 mnemonic (SEP-0005).
    pub fn derive_public_key(&self, mnemonic: String, index: u32) -> Result<String, SignerError> {
        Ok(Keypair::from_mnemonic(&mnemonic, index)?.public_key())
    }

    /// Get public key (G...) from secret key (S...).
    pub fn public_key_from_secret(&self, secret: String) -> Result<String, SignerError> {
        Ok(Keypair::from_secret(&secret)?.public_key())
    }

    /// Read vault configuration from chain.
    pub fn get_vault_config(
        &self,
        vault: String,
        network: String,
        rpc_url: Option<String>,
    ) -> Result<FfiVaultConfig, SignerError> {
        block_on(async {
            let client = client(&vault, &network, rpc_url)?;
            let cfg = client.reader().get_config().await?;
            Ok(FfiVaultConfig::from(cfg))
        })
    }

    /// List vault signer addresses.
    pub fn get_vault_signers(
        &self,
        vault: String,
        network: String,
        rpc_url: Option<String>,
    ) -> Result<Vec<String>, SignerError> {
        block_on(async {
            let client = client(&vault, &network, rpc_url)?;
            client.reader().get_signers().await.map_err(SignerError::from)
        })
    }

    /// Read on-chain proposal status.
    pub fn get_proposal(
        &self,
        vault: String,
        network: String,
        proposal_id: u64,
        rpc_url: Option<String>,
    ) -> Result<FfiProposal, SignerError> {
        block_on(async {
            let client = client(&vault, &network, rpc_url)?;
            let proposal = client.reader().get_proposal(proposal_id).await?;
            Ok(FfiProposal::from(proposal))
        })
    }

    /// List proposals awaiting approval (on-chain status = pending).
    pub fn list_pending_proposals(
        &self,
        vault: String,
        network: String,
        rpc_url: Option<String>,
    ) -> Result<Vec<FfiProposalSummary>, SignerError> {
        block_on(async {
            let client = client(&vault, &network, rpc_url)?;
            let proposals = client.reader().list_pending_proposals().await?;
            Ok(proposals.into_iter().map(FfiProposalSummary::from).collect())
        })
    }

    /// Build unsigned approve transaction XDR (base64 envelope).
    pub fn build_approve_tx(
        &self,
        vault: String,
        network: String,
        signer: String,
        proposal_id: u64,
        rpc_url: Option<String>,
    ) -> Result<String, SignerError> {
        block_on(async {
            let client = client(&vault, &network, rpc_url)?;
            client
                .writer()
                .build_approve_tx(&signer, proposal_id)
                .await
                .map_err(SignerError::from)
        })
    }

    /// Sign an unsigned transaction envelope XDR with a secret key.
    pub fn sign_transaction(
        &self,
        unsigned_xdr: String,
        secret: String,
        network: String,
    ) -> Result<String, SignerError> {
        let net = Network::parse(&network)?;
        let kp = Keypair::from_secret(&secret)?;
        sign_transaction_xdr(&unsigned_xdr, net, &kp).map_err(SignerError::from)
    }

    /// Submit a signed transaction envelope XDR; returns tx hash.
    pub fn submit_transaction(
        &self,
        signed_xdr: String,
        network: String,
        rpc_url: Option<String>,
    ) -> Result<String, SignerError> {
        block_on(async {
            let net = Network::parse(&network)?;
            let url = rpc_url_for(&network, rpc_url)?;
            submit_signed_xdr(&url, net, &signed_xdr)
                .await
                .map_err(SignerError::from)
        })
    }

    /// Approve a proposal: build, sign, submit, and wait for confirmation.
    pub fn approve_proposal(
        &self,
        vault: String,
        network: String,
        secret: String,
        proposal_id: u64,
        rpc_url: Option<String>,
    ) -> Result<String, SignerError> {
        block_on(async {
            let net = Network::parse(&network)?;
            let url = rpc_url_for(&network, rpc_url)?;
            let kp = Keypair::from_secret(&secret)?;
            let client = VaultClient::new(&vault, &url, net)?;
            let hash = client.writer().approve(&kp, proposal_id).await?;
            poll_transaction(&url, &hash).await?;
            Ok(hash)
        })
    }

    /// Reject a proposal: build, sign, submit, and wait for confirmation.
    pub fn reject_proposal(
        &self,
        vault: String,
        network: String,
        secret: String,
        proposal_id: u64,
        rpc_url: Option<String>,
    ) -> Result<String, SignerError> {
        block_on(async {
            let net = Network::parse(&network)?;
            let url = rpc_url_for(&network, rpc_url)?;
            let kp = Keypair::from_secret(&secret)?;
            let client = VaultClient::new(&vault, &url, net)?;
            let hash = client.writer().reject(&kp, proposal_id).await?;
            poll_transaction(&url, &hash).await?;
            Ok(hash)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_vault() -> Option<String> {
        std::env::var("VAULT_ADDRESS").ok()
    }

    #[test]
    fn ffi_reads_vault_config_when_env_set() {
        let Some(vault) = test_vault() else {
            eprintln!("skip: set VAULT_ADDRESS to run integration test");
            return;
        };
        let signer = VaultSigner::new();
        let cfg = signer
            .get_vault_config(vault, "testnet".to_string(), None)
            .expect("get_vault_config");
        assert!(!cfg.name.is_empty());
        assert!(cfg.threshold >= 1);
    }
}
