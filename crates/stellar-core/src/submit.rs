use {
    crate::{keypair::Keypair, network::Network, prepare::rpc_server},
    anyhow::{anyhow, Result},
    soroban_client::{
        soroban_rpc::{SendTransactionStatus, TransactionStatus},
        transaction::TransactionBehavior,
        xdr::{Limits, WriteXdr},
    },
    std::time::Duration,
};

const POLL_ATTEMPTS: usize = 15;
const POLL_INTERVAL: Duration = Duration::from_secs(2);

pub fn sign_transaction_xdr(unsigned_xdr: &str, network: Network, keypair: &Keypair) -> Result<String> {
    let mut tx = stellar_baselib::transaction::Transaction::from_xdr_envelope(unsigned_xdr, network.passphrase());
    tx.sign(&[keypair.inner().clone()]);
    tx.to_envelope()
        .map_err(|e| anyhow!("to_envelope: {}", e))?
        .to_xdr_base64(Limits::none())
        .map_err(|e| anyhow!("XDR encode: {:?}", e))
}

pub async fn sign_and_submit(rpc_url: &str, network: Network, keypair: &Keypair, unsigned_xdr: &str) -> Result<String> {
    let signed_xdr = sign_transaction_xdr(unsigned_xdr, network, keypair)?;
    submit_signed_xdr(rpc_url, network, &signed_xdr).await
}

async fn submit_signed_xdr(rpc_url: &str, network: Network, signed_xdr: &str) -> Result<String> {
    let tx = stellar_baselib::transaction::Transaction::from_xdr_envelope(signed_xdr, network.passphrase());
    let server = rpc_server(rpc_url)?;
    let result = server
        .send_transaction(tx)
        .await
        .map_err(|e| anyhow!("send_transaction: {:?}", e))?;

    if result.status == SendTransactionStatus::Error {
        let detail = result
            .to_error_result()
            .map(|r| format!("{:?}", r))
            .unwrap_or_else(|| "unknown".to_string());
        return Err(anyhow!("transaction rejected: {}", detail));
    }

    Ok(result.hash)
}

pub async fn poll_transaction(rpc_url: &str, hash: &str) -> Result<()> {
    let server = rpc_server(rpc_url)?;
    for _ in 0..POLL_ATTEMPTS {
        tokio::time::sleep(POLL_INTERVAL).await;
        match server.get_transaction(hash).await {
            Ok(status) => {
                if status.status == TransactionStatus::Success {
                    return Ok(());
                }
                if status.status == TransactionStatus::Failed {
                    let reason = status
                        .to_result()
                        .map(|r| format!("{:?}", r))
                        .unwrap_or_else(|| "unknown".to_string());
                    return Err(anyhow!("transaction failed on-chain: {}", reason));
                }
            }
            Err(e) => {
                eprintln!("poll warning: {}", e);
            }
        }
    }
    Err(anyhow!("transaction poll timeout"))
}
