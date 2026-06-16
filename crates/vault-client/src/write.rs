use {
    crate::{config::DEFAULT_WRITE_FEE, VaultClient},
    anyhow::{anyhow, Result},
    soroban_client::{
        contract::{ContractBehavior, Contracts},
        transaction::{AccountBehavior, TransactionBehavior},
        transaction_builder::{TransactionBuilder, TransactionBuilderBehavior, TIMEOUT_INFINITE},
        xdr::{Limits, WriteXdr},
    },
    stellar_core::{address_to_scval, fetch_account_sequence, u64_to_scval, Keypair},
};

pub struct VaultWriter<'a> {
    pub client: &'a VaultClient,
}

impl VaultWriter<'_> {
    /// Build unsigned approve XDR (base64 envelope) for external signing.
    pub async fn build_approve_tx(&self, signer: &str, proposal_id: u64) -> Result<String> {
        self.build_contract_tx(
            signer,
            "approve",
            vec![address_to_scval(signer)?, u64_to_scval(proposal_id)],
            DEFAULT_WRITE_FEE,
        )
        .await
    }

    /// Build unsigned reject XDR.
    pub async fn build_reject_tx(&self, signer: &str, proposal_id: u64) -> Result<String> {
        self.build_contract_tx(
            signer,
            "reject",
            vec![address_to_scval(signer)?, u64_to_scval(proposal_id)],
            DEFAULT_WRITE_FEE,
        )
        .await
    }

    /// Sign and submit an approve transaction.
    pub async fn approve(&self, keypair: &Keypair, proposal_id: u64) -> Result<String> {
        use stellar_core::sign_and_submit;

        let signer = keypair.public_key();
        let unsigned = self.build_approve_tx(&signer, proposal_id).await?;
        sign_and_submit(&self.client.rpc_url, self.client.network, keypair, &unsigned).await
    }

    /// Sign and submit a reject transaction.
    pub async fn reject(&self, keypair: &Keypair, proposal_id: u64) -> Result<String> {
        use stellar_core::sign_and_submit;

        let signer = keypair.public_key();
        let unsigned = self.build_reject_tx(&signer, proposal_id).await?;
        sign_and_submit(&self.client.rpc_url, self.client.network, keypair, &unsigned).await
    }

    async fn build_contract_tx(
        &self,
        signer: &str,
        function: &str,
        args: Vec<soroban_client::xdr::ScVal>,
        fee: u32,
    ) -> Result<String> {
        let contract = Contracts::new(&self.client.vault).map_err(|e| anyhow!("invalid vault contract: {}", e))?;
        let op = contract.call(function, Some(args));

        let sequence = fetch_account_sequence(&self.client.horizon_url, signer).await?;
        let mut account = soroban_client::account::Account::new(signer, &sequence.to_string())
            .map_err(|e| anyhow!("invalid account: {}", e))?;

        let tx = TransactionBuilder::new(&mut account, self.client.network.passphrase(), None)
            .fee(fee)
            .add_operation(op)
            .set_timeout(TIMEOUT_INFINITE)
            .map_err(|e| anyhow!("timeout: {}", e))?
            .build();

        let prepared = self.client.rpc.prepare_transaction(&tx).await?;
        let envelope = prepared.to_envelope().map_err(|e| anyhow!("to_envelope: {}", e))?;

        envelope
            .to_xdr_base64(Limits::none())
            .map_err(|e| anyhow!("XDR encode: {:?}", e))
    }
}
