#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Network {
    Testnet,
    Mainnet,
}

impl Network {
    pub fn passphrase(self) -> &'static str {
        match self {
            Network::Testnet => "Test SDF Network ; September 2015",
            Network::Mainnet => "Public Global Stellar Network ; September 2015",
        }
    }

    pub fn default_rpc_url(self) -> &'static str {
        match self {
            Network::Testnet => "https://soroban-testnet.stellar.org",
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
