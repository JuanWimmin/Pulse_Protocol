use crate::services::soroban::{
    address_to_scval, bytes_to_scval, symbol_to_scval, u32_to_scval, SorobanClient,
};
use tracing::info;

/// Typed wrapper for the ProofOfLife smart contract on Soroban.
#[derive(Clone)]
pub struct ProofOfLifeContractClient {
    contract_id: String,
    soroban: SorobanClient,
}

impl ProofOfLifeContractClient {
    pub fn new(contract_id: &str, soroban: SorobanClient) -> Self {
        Self {
            contract_id: contract_id.to_string(),
            soroban,
        }
    }

    /// Submit a proof-of-life verification score on-chain.
    /// This is the core function called by the oracle/publisher.
    pub async fn submit_verification(
        &self,
        user: &str,
        score: u32,
        source: &str,
        oracle_signature: &[u8],
    ) -> Result<String, String> {
        let args = vec![
            address_to_scval(user)?,
            u32_to_scval(score),
            symbol_to_scval(source)?,
            bytes_to_scval(oracle_signature)?,
        ];

        let tx_hash = self
            .soroban
            .invoke_contract(&self.contract_id, "submit_verification", args)
            .await?;

        info!("submit_verification tx: {} (user={}, score={})", tx_hash, user, score);
        Ok(tx_hash)
    }

    /// Emergency check-in: user proves they are alive, resets to ACTIVE.
    pub async fn emergency_checkin(&self, user: &str) -> Result<String, String> {
        let args = vec![address_to_scval(user)?];

        let tx_hash = self
            .soroban
            .invoke_contract(&self.contract_id, "emergency_checkin", args)
            .await?;

        info!("emergency_checkin tx: {} (user={})", tx_hash, user);
        Ok(tx_hash)
    }

    /// Link a vault contract to this proof-of-life instance.
    pub async fn link_vault(
        &self,
        admin: &str,
        user: &str,
        vault_contract: &str,
        vault_id: u32,
    ) -> Result<String, String> {
        let args = vec![
            address_to_scval(admin)?,
            address_to_scval(user)?,
            address_to_scval(vault_contract)?,
            u32_to_scval(vault_id),
        ];

        self.soroban
            .invoke_contract(&self.contract_id, "link_vault", args)
            .await
    }

    /// Register the oracle model (admin only, typically done at deploy time).
    pub async fn register_model(
        &self,
        admin: &str,
        oracle: &str,
        threshold: u32,
    ) -> Result<String, String> {
        let args = vec![
            address_to_scval(admin)?,
            address_to_scval(oracle)?,
            u32_to_scval(threshold),
        ];

        self.soroban
            .invoke_contract(&self.contract_id, "register_model", args)
            .await
    }
}
