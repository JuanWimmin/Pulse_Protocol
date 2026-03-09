use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    /// Soroban RPC endpoint (testnet default).
    pub stellar_rpc_url: String,
    /// Stellar network passphrase.
    pub stellar_network_passphrase: String,
    /// Oracle secret key (Stellar S... format) for signing on-chain transactions.
    pub oracle_secret_key: Option<String>,
    /// Deployed contract IDs (set after deploy).
    pub vault_contract_id: Option<String>,
    pub proof_of_life_contract_id: Option<String>,
    pub beneficiary_contract_id: Option<String>,
    /// Trustless Work factory contract ID (backend-orquesta mode).
    pub tw_factory_contract_id: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
            host: env::var("HOST")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("PORT must be a number"),
            stellar_rpc_url: env::var("STELLAR_RPC_URL")
                .unwrap_or_else(|_| "https://soroban-testnet.stellar.org".to_string()),
            stellar_network_passphrase: env::var("STELLAR_NETWORK_PASSPHRASE")
                .unwrap_or_else(|_| "Test SDF Network ; September 2015".to_string()),
            oracle_secret_key: env::var("ORACLE_SECRET_KEY").ok(),
            vault_contract_id: env::var("VAULT_CONTRACT_ID").ok(),
            proof_of_life_contract_id: env::var("PROOF_OF_LIFE_CONTRACT_ID").ok(),
            beneficiary_contract_id: env::var("BENEFICIARY_CONTRACT_ID").ok(),
            tw_factory_contract_id: env::var("TW_FACTORY_CONTRACT_ID").ok(),
        }
    }
}