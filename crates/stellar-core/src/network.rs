#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Network {
    Testnet,
    Mainnet,
}

impl Network {
    pub fn parse(s: &str) -> anyhow::Result<Self> {
        match s.to_lowercase().as_str() {
            "testnet" | "test" => Ok(Network::Testnet),
            "mainnet" | "public" => Ok(Network::Mainnet),
            other => Err(anyhow::anyhow!("unknown network: {} (use testnet or mainnet)", other)),
        }
    }

    pub fn passphrase(self) -> &'static str {
        match self {
            Network::Testnet => "Test SDF Network ; September 2015",
            Network::Mainnet => "Public Global Stellar Network ; September 2015",
        }
    }

    pub fn default_rpc_url(self) -> &'static str {
        match self {
            // Official SDF RPC is unreachable in some regions; Gateway.fm is a reliable public fallback.
            Network::Testnet => "https://soroban-rpc.testnet.stellar.gateway.fm",
            Network::Mainnet => "https://soroban.stellar.org",
        }
    }

    pub fn default_horizon_url(self) -> &'static str {
        match self {
            Network::Testnet => "https://horizon-testnet.stellar.org",
            Network::Mainnet => "https://horizon.stellar.org",
        }
    }
}
