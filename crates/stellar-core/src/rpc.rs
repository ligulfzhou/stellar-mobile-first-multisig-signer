use {
    crate::network::Network,
    anyhow::{anyhow, Result},
    soroban_client::{
        contract::{ContractBehavior, Contracts},
        soroban_rpc::SimulateTransactionResponse,
        transaction::AccountBehavior,
        transaction_builder::{TransactionBuilder, TransactionBuilderBehavior},
        xdr::{self, LedgerEntryData, LedgerKey, LedgerKeyAccount},
        Options, Server,
    },
    stellar_baselib::transaction::Transaction,
};

pub struct RpcClient {
    server: Server,
    network: Network,
}

impl RpcClient {
    pub fn new(rpc_url: &str, network: Network) -> Result<Self> {
        let server = Server::new(
            rpc_url,
            Options {
                allow_http: true,
                ..Default::default()
            },
        )
        .map_err(|e| anyhow!("create RPC client: {:?}", e))?;
        Ok(Self { server, network })
    }

    pub fn network(&self) -> Network {
        self.network
    }

    pub fn server(&self) -> &Server {
        &self.server
    }

    /// Read-only contract call via RPC simulate.
    pub async fn simulate_contract_call(
        &self,
        contract_address: &str,
        function_name: &str,
        args: Vec<xdr::ScVal>,
    ) -> Result<xdr::ScVal> {
        let contract = Contracts::new(contract_address).map_err(|e| anyhow!("invalid contract: {}", e))?;
        let op = contract.call(function_name, if args.is_empty() { None } else { Some(args) });

        let mut dummy =
            soroban_client::account::Account::new("GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF", "0")
                .map_err(|e| anyhow!("dummy account: {:?}", e))?;

        let tx = TransactionBuilder::new(&mut dummy, self.network.passphrase(), None)
            .fee(100u32)
            .add_operation(op)
            .set_timeout(30)
            .map_err(|e| anyhow!("timeout: {:?}", e))?
            .build();

        let sim = self
            .server
            .simulate_transaction(&tx, None)
            .await
            .map_err(|e| anyhow!("simulate_transaction: {:?}", e))?;

        if let Some(err) = &sim.error {
            return Err(anyhow!("simulation error: {}", err));
        }

        let (scval, _auth) = sim
            .to_result()
            .ok_or_else(|| anyhow!("simulation returned no result"))?;

        Ok(scval)
    }

    pub async fn prepare_transaction(&self, tx: &Transaction) -> Result<Transaction> {
        self.server
            .prepare_transaction(tx)
            .await
            .map_err(|e| anyhow!("prepare_transaction: {:?}", e))
    }

    pub async fn simulate_transaction(&self, tx: &Transaction) -> Result<SimulateTransactionResponse> {
        self.server
            .simulate_transaction(tx, None)
            .await
            .map_err(|e| anyhow!("simulate_transaction: {:?}", e))
    }

    pub async fn get_sequence(&self, public_key: &str) -> Result<u64> {
        let account_id = stellar_strkey::ed25519::PublicKey::from_string(public_key)
            .map(|pk| xdr::AccountId(xdr::PublicKey::PublicKeyTypeEd25519(xdr::Uint256(pk.0))))
            .map_err(|e| anyhow!("invalid public key: {}", e))?;

        let ledger_key = LedgerKey::Account(LedgerKeyAccount { account_id });
        let resp = self
            .server
            .get_ledger_entries(vec![ledger_key])
            .await
            .map_err(|e| anyhow!("get_ledger_entries: {:?}", e))?;

        let entries = resp
            .entries
            .ok_or_else(|| anyhow!("account not found: {}", public_key))?;
        let entry = entries.into_iter().next().ok_or_else(|| anyhow!("account not found"))?;

        match entry.to_data() {
            LedgerEntryData::Account(account) => Ok(Into::<i64>::into(account.seq_num) as u64),
            _ => Err(anyhow!("unexpected ledger entry type")),
        }
    }
}
