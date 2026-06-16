use {
    crate::{config::DEFAULT_WRITE_FEE, tx::build_invoke_tx, VaultClient},
    anyhow::Result,
    soroban_client::xdr::ScVal,
    stellar_core::{
        address_to_scval, bool_to_scval, i128_to_scval, sign_and_submit, symbol_to_scval, u32_to_scval,
        u64_to_scval, Keypair,
    },
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

    /// Propose a native/token transfer (ProposalType::Transfer = 0).
    pub async fn propose_transfer(
        &self,
        keypair: &Keypair,
        token: &str,
        recipient: &str,
        amount: i128,
    ) -> Result<String> {
        use stellar_core::sign_and_submit;

        let proposer = keypair.public_key();
        let unsigned = self
            .build_propose_transfer_tx(&proposer, token, recipient, amount)
            .await?;
        sign_and_submit(&self.client.rpc_url, self.client.network, keypair, &unsigned).await
    }

    pub async fn build_propose_transfer_tx(
        &self,
        proposer: &str,
        token: &str,
        recipient: &str,
        amount: i128,
    ) -> Result<String> {
        self.build_contract_tx(
            proposer,
            "propose",
            vec![
                address_to_scval(proposer)?,
                u32_to_scval(0), // Transfer
                address_to_scval(token)?,
                address_to_scval(recipient)?,
                i128_to_scval(amount),
                u64_to_scval(0),
                u64_to_scval(0),
                u64_to_scval(0),
                u64_to_scval(0),
                bool_to_scval(false),
                symbol_to_scval("transfer")?,
            ],
            DEFAULT_WRITE_FEE,
        )
        .await
    }

    /// Execute an approved transfer proposal.
    pub async fn execute_transfer(
        &self,
        keypair: &Keypair,
        proposal_id: u64,
        token: &str,
        recipient: &str,
        amount: i128,
    ) -> Result<String> {
        use stellar_core::sign_and_submit;

        let executor = keypair.public_key();
        let unsigned = self
            .build_execute_transfer_tx(&executor, proposal_id, token, recipient, amount)
            .await?;
        sign_and_submit(&self.client.rpc_url, self.client.network, keypair, &unsigned).await
    }

    pub async fn build_execute_transfer_tx(
        &self,
        executor: &str,
        proposal_id: u64,
        token: &str,
        recipient: &str,
        amount: i128,
    ) -> Result<String> {
        self.build_contract_tx(
            executor,
            "execute",
            vec![
                address_to_scval(executor)?,
                u64_to_scval(proposal_id),
                u32_to_scval(0), // Transfer
                address_to_scval(token)?,
                address_to_scval(recipient)?,
                i128_to_scval(amount),
                u64_to_scval(0),
                u64_to_scval(0),
                u64_to_scval(0),
                u64_to_scval(0),
                bool_to_scval(false),
            ],
            DEFAULT_WRITE_FEE,
        )
        .await
    }

    async fn build_contract_tx(
        &self,
        signer: &str,
        function: &str,
        args: Vec<ScVal>,
        fee: u32,
    ) -> Result<String> {
        build_invoke_tx(
            &self.client.rpc,
            self.client.network,
            &self.client.horizon_url,
            &self.client.vault,
            signer,
            function,
            args,
            fee,
        )
        .await
    }
}
