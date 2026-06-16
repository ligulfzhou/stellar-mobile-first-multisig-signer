use {
    anyhow::{anyhow, Result},
    soroban_client::{
        contract::{ContractBehavior, Contracts},
        transaction::{AccountBehavior, TransactionBehavior},
        transaction_builder::{TransactionBuilder, TransactionBuilderBehavior, TIMEOUT_INFINITE},
        xdr::{Limits, ScVal, WriteXdr},
    },
    stellar_core::{fetch_account_sequence, Network, RpcClient},
};

/// Build an unsigned Soroban invoke envelope (base64 XDR).
pub async fn build_invoke_tx(
    rpc: &RpcClient,
    network: Network,
    horizon_url: &str,
    contract_id: &str,
    signer: &str,
    function: &str,
    args: Vec<ScVal>,
    fee: u32,
) -> Result<String> {
    let contract = Contracts::new(contract_id).map_err(|e| anyhow!("invalid contract: {}", e))?;
    let op = contract.call(function, Some(args));

    let sequence = match rpc.get_sequence(signer).await {
        Ok(seq) => seq,
        Err(_) => fetch_account_sequence(horizon_url, signer).await?,
    };
    let mut account = soroban_client::account::Account::new(signer, &sequence.to_string())
        .map_err(|e| anyhow!("invalid account: {}", e))?;

    let tx = TransactionBuilder::new(&mut account, network.passphrase(), None)
        .fee(fee)
        .add_operation(op)
        .set_timeout(TIMEOUT_INFINITE)
        .map_err(|e| anyhow!("timeout: {}", e))?
        .build();

    let prepared = rpc.prepare_transaction(&tx).await?;
    let envelope = prepared.to_envelope().map_err(|e| anyhow!("to_envelope: {}", e))?;

    envelope
        .to_xdr_base64(Limits::none())
        .map_err(|e| anyhow!("XDR encode: {:?}", e))
}
