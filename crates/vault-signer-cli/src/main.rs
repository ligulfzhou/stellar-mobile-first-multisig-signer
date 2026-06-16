use {
    anyhow::Result,
    clap::{Parser, Subcommand},
    stellar_core::{network::Network, poll_transaction, Keypair},
    vault_client::VaultClient,
};

#[derive(Parser)]
#[command(name = "vault-signer", about = "Multisig vault signer CLI")]
struct Cli {
    /// Soroban RPC URL
    #[arg(long, env = "SOROBAN_RPC_URL")]
    rpc_url: Option<String>,

    /// Network: testnet or mainnet
    #[arg(long, default_value = "testnet")]
    network: String,

    /// Vault contract address (C...)
    #[arg(long, env = "VAULT_ADDRESS")]
    vault: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show vault configuration
    Config,
    /// List vault signers
    Signers,
    /// List all proposals
    Proposals {
        #[arg(long)]
        pending_only: bool,
    },
    /// Show proposal status
    Proposal {
        #[arg(long)]
        id: u64,
    },
    /// Approve a proposal (requires --secret)
    Approve {
        #[arg(long)]
        id: u64,
        /// Stellar secret key (S...)
        #[arg(long, env = "STELLAR_SECRET")]
        secret: String,
    },
    /// Reject a proposal (requires --secret)
    Reject {
        #[arg(long)]
        id: u64,
        #[arg(long, env = "STELLAR_SECRET")]
        secret: String,
    },
    /// Propose a token transfer (requires --secret)
    Propose {
        /// Token contract (native XLM SAC on testnet by default)
        #[arg(long, default_value = "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC")]
        token: String,
        #[arg(long)]
        recipient: String,
        /// Amount in stroops (1 XLM = 10_000_000)
        #[arg(long)]
        amount: i128,
        #[arg(long, env = "STELLAR_SECRET")]
        secret: String,
    },
    /// Execute an approved proposal (requires --secret)
    Execute {
        #[arg(long)]
        id: u64,
        #[arg(long, default_value = "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC")]
        token: String,
        #[arg(long)]
        recipient: String,
        #[arg(long)]
        amount: i128,
        #[arg(long, env = "STELLAR_SECRET")]
        secret: String,
    },
    /// Derive public key from mnemonic (dev helper)
    DeriveKey {
        #[arg(long)]
        mnemonic: String,
        #[arg(long, default_value = "0")]
        index: u32,
    },
}

fn parse_network(s: &str) -> Result<Network> {
    match s.to_lowercase().as_str() {
        "testnet" | "test" => Ok(Network::Testnet),
        "mainnet" | "public" => Ok(Network::Mainnet),
        other => anyhow::bail!("unknown network: {} (use testnet or mainnet)", other),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let network = parse_network(&cli.network)?;
    let rpc_url = cli.rpc_url.unwrap_or_else(|| network.default_rpc_url().to_string());

    let client = VaultClient::new(cli.vault, rpc_url.clone(), network)?;

    match cli.command {
        Commands::Config => {
            let cfg = client.reader().get_config().await?;
            println!("Vault: {}", client.vault);
            println!("Name: {}", cfg.name);
            println!("Threshold: {}", cfg.threshold);
            println!("Signers: {}", cfg.signer_count);
            println!("Proposals: {}", cfg.proposal_count);
            println!("Locks: {}", cfg.lock_count);
            println!("Fee amount: {}", cfg.fee_amount);
        }
        Commands::Signers => {
            let signers = client.reader().get_signers().await?;
            println!("Signers ({}):", signers.len());
            for (i, s) in signers.iter().enumerate() {
                println!("  {}. {}", i + 1, s);
            }
        }
        Commands::Proposals { pending_only } => {
            let proposals = if pending_only {
                client.reader().list_pending_proposals().await?
            } else {
                client.reader().list_proposals().await?
            };
            if proposals.is_empty() {
                println!("No proposals.");
            } else {
                for p in proposals {
                    println!(
                        "#{} {:?} approvals={} rejections={} status={}",
                        p.id, p.proposal_type, p.approval_count, p.rejection_count, p.status
                    );
                }
            }
        }
        Commands::Proposal { id } => {
            let p = client.reader().get_proposal(id).await?;
            println!("Proposal #{}", id);
            println!("  type: {:?}", p.proposal_type);
            println!("  approvals: {}", p.approval_count);
            println!("  rejections: {}", p.rejection_count);
            println!("  status: {}", p.status_label());
        }
        Commands::Approve { id, secret } => {
            let kp = Keypair::from_secret(&secret)?;
            println!("Signer: {}", kp.public_key());
            let hash = client.writer().approve(&kp, id).await?;
            println!("Submitted: {}", hash);
            poll_transaction(&rpc_url, &hash).await?;
            println!("Confirmed on-chain.");
        }
        Commands::Reject { id, secret } => {
            let kp = Keypair::from_secret(&secret)?;
            println!("Signer: {}", kp.public_key());
            let hash = client.writer().reject(&kp, id).await?;
            println!("Submitted: {}", hash);
            poll_transaction(&rpc_url, &hash).await?;
            println!("Confirmed on-chain.");
        }
        Commands::Propose {
            token,
            recipient,
            amount,
            secret,
        } => {
            let kp = Keypair::from_secret(&secret)?;
            println!("Proposer: {}", kp.public_key());
            let hash = client
                .writer()
                .propose_transfer(&kp, &token, &recipient, amount)
                .await?;
            println!("Submitted: {}", hash);
            poll_transaction(&rpc_url, &hash).await?;
            let cfg = client.reader().get_config().await?;
            println!("Proposal #{} created.", cfg.proposal_count);
        }
        Commands::Execute {
            id,
            token,
            recipient,
            amount,
            secret,
        } => {
            let kp = Keypair::from_secret(&secret)?;
            println!("Executor: {}", kp.public_key());
            let hash = client
                .writer()
                .execute_transfer(&kp, id, &token, &recipient, amount)
                .await?;
            println!("Submitted: {}", hash);
            poll_transaction(&rpc_url, &hash).await?;
            println!("Transfer executed on-chain.");
        }
        Commands::DeriveKey { mnemonic, index } => {
            let kp = Keypair::from_mnemonic(&mnemonic, index)?;
            println!("Public key: {}", kp.public_key());
        }
    }

    Ok(())
}
