use {
    crate::network::Network,
    anyhow::{anyhow, Result},
    soroban_client::{
        transaction::{AccountBehavior, TransactionBehavior},
        transaction_builder::{TransactionBuilder, TransactionBuilderBehavior, TIMEOUT_INFINITE},
        xdr::{self, Limits, ReadXdr, WriteXdr},
        Options, Server,
    },
    stellar_xdr::{
        curr as sxdr,
        curr::{Limits as StellarLimits, WriteXdr as StellarWriteXdr},
    },
};

pub fn rpc_server(rpc_url: &str) -> Result<Server> {
    Server::new(
        rpc_url,
        Options {
            allow_http: true,
            ..Default::default()
        },
    )
    .map_err(|e| anyhow!("create Soroban RPC client: {}", e))
}

pub async fn fetch_account_sequence(horizon_url: &str, public_key: &str) -> Result<u64> {
    let url = format!("{}/accounts/{}", horizon_url.trim_end_matches('/'), public_key);
    let data: serde_json::Value = reqwest::Client::new()
        .get(&url)
        .send()
        .await
        .map_err(|e| anyhow!("Horizon account request: {}", e))?
        .json()
        .await
        .map_err(|e| anyhow!("Horizon account JSON: {}", e))?;

    let seq_str = data
        .get("sequence")
        .and_then(|s| s.as_str())
        .ok_or_else(|| anyhow!("missing sequence in Horizon response"))?;

    seq_str.parse::<u64>().map_err(|e| anyhow!("parse sequence: {}", e))
}

/// Simulate + assemble and return unsigned envelope XDR (base64).
pub async fn prepare_transaction_xdr(
    rpc_url: &str,
    network: Network,
    public_key: &str,
    sequence: u64,
    operations: &[sxdr::Operation],
    fee: u32,
) -> Result<String> {
    let mut account = soroban_client::account::Account::new(public_key, &sequence.to_string())
        .map_err(|e| anyhow!("invalid account/sequence: {}", e))?;

    let mut builder = TransactionBuilder::new(&mut account, network.passphrase(), None);
    builder.fee(fee);

    for op in operations {
        let op_bytes = op
            .to_xdr(StellarLimits::none())
            .map_err(|e| anyhow!("encode operation: {:?}", e))?;
        let client_op =
            xdr::Operation::from_xdr(op_bytes, Limits::none()).map_err(|e| anyhow!("decode operation: {:?}", e))?;
        builder.add_operation(client_op);
    }

    let tx = builder
        .set_timeout(TIMEOUT_INFINITE)
        .map_err(|e| anyhow!("timeout: {:?}", e))?
        .build();

    let server = rpc_server(rpc_url)?;
    let prepared = server
        .prepare_transaction(&tx)
        .await
        .map_err(|e| anyhow!("prepare_transaction: {:?}", e))?;

    let envelope = prepared.to_envelope().map_err(|e| anyhow!("to_envelope: {}", e))?;

    envelope
        .to_xdr_base64(Limits::none())
        .map_err(|e| anyhow!("XDR encode: {:?}", e))
}
